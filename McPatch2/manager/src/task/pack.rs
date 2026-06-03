use std::ops::Deref;
use std::future::Future;
use std::rc::Weak;

use crate::app_path::AppPath;
use crate::config::Config;
use crate::core::archive_tester::ArchiveTester;
use crate::core::data::index_file::IndexFile;
use crate::core::data::index_file::VersionIndex;
use crate::core::data::version_meta::VersionMeta;
use crate::core::data::version_meta_group::VersionMetaGroup;
use crate::core::tar_writer::TarWriter;
use crate::core::update_signature::sign_version_index;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::log::Console;

fn block_on_future<F>(future: F) -> F::Output
where
    F: Future,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        tokio::runtime::Runtime::new().unwrap().block_on(future)
    }
}

pub fn task_pack(version_label: String, change_logs: String, apppath: &AppPath, config: &Config, console: &Console) -> u8 {
    // 读取更新日志
    let change_logs = match change_logs.is_empty() {
        false => change_logs,
        true => {
            // 如果没有指定更新日志，就从文件里读取
            let logs_file = apppath.working_dir.join("logs.txt");

            match std::fs::read_to_string(&logs_file) {
                Ok(text) => text,
                Err(_) => "没有更新记录".to_owned(),
            }
        },
    };


    let mut index_file = IndexFile::load_from_file(&apppath.index_file);

    if index_file.contains(&version_label) {
        console.log_error(format!("版本号已经存在: {}", version_label));
        return 1;
    }

    // 1. 读取所有历史版本，并推演出上个版本的文件状态，用于和工作空间目录对比生成文件差异
    // 读取现有更新包，并复现在history上
    console.log_debug("正在读取数据");

    let mut history = HistoryFile::new_dir("workspace_root", Weak::new());

    for (_index, meta) in index_file.read_all_metas(&apppath.public_dir) {
        history.replay_operations(&meta);
    }

    // 对比文件
    console.log_debug("正在扫描文件更改");

    let exclude_rules = &config.core.exclude_rules;
    let disk_file = DiskFile::new(apppath.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(exclude_rules));

    if !diff.has_diff() {
        console.log_error("目前工作目录还没有任何文件修改");
        return 1;
    }

    console.log_info(format!("{:#?}", diff));

    // 2. 将所有“覆盖的文件”的数据和元数据写入到更新包中，同时更新元数据中每个文件的偏移值
    // 创建新的更新包，将所有文件修改写进去
    std::fs::create_dir_all(&apppath.public_dir).unwrap();
    let version_filename = format!("{}.tar", version_label);
    let version_file = apppath.public_dir.join(&version_filename);
    let mut writer = TarWriter::new(&version_file);

    // 写入每个更新的文件数据
    let mut vec = Vec::<&DiskFile>::new();
    
    for f in &diff.added_files {
        vec.push(f);
    }

    for f in &diff.modified_files {
        vec.push(f);
    }

    let mut counter = 1;
    for f in &vec {
        console.log_debug(format!("打包({}/{}) {}", counter, vec.len(), f.path().deref()));
        counter += 1;

        let path = f.path().to_owned();
        let disk_file = apppath.workspace_dir.join(&path);
        let open = std::fs::File::options().read(true).open(disk_file).unwrap();

        // 提供的len必须和读取到的长度严格相等
        let meta = open.metadata().unwrap();
        assert_eq!(meta.len(), f.len());

        writer.add_file(open, f.len(), &path, &version_label);
    }

    // 写入元数据
    console.log_debug("写入元数据");

    // 读取写好的更新记录
    let meta = VersionMeta::new(version_label.clone(), change_logs, diff.to_file_changes());
    let meta_group = VersionMetaGroup::with_one(meta);
    let meta_info = writer.finish(meta_group);

    // 3. 更新索引文件
    let mut version_index = VersionIndex {
        label: version_label.to_owned(),
        filename: version_filename,
        offset: meta_info.offset,
        len: meta_info.length,
        hash: meta_info.hash,
        signature: String::new(),
    };
    version_index.signature = match sign_version_index(&version_index, &config.core.index_signature_private_key) {
        Ok(signature) => signature,
        Err(err) => {
            console.log_error(format!("更新索引签名失败: {}", err));
            return 1;
        }
    };
    index_file.add(version_index);

    // 进行解压测试
    console.log_debug("正在测试");

    let mut tester = ArchiveTester::new();
    for (index, meta) in index_file.read_all_metas(&apppath.public_dir) {
        tester.feed_version(apppath.public_dir.join(&index.filename), &meta);
    }
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();

    // 4. 模组识别（ModRouter）
    if let Some(router) = crate::task::mod_router::ModRouter::new(config.core.mod_platform.clone()) {
        console.log_info("模组平台识别已启用");
        for f in &vec {
            let path = f.path().to_owned();
            let ext = std::path::Path::new(&path).extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "jar" || ext == "zip" {
                match block_on_future(router.resolve_file_source(f)) {
                    Ok(Some(source)) => {
                        console.log_info(format!("  识别为: {} (来自 {})", source.mod_name, source.platform));
                    }
                    Ok(None) => {}
                    Err(e) => {
                        console.log_debug(format!("  识别失败: {:?}", e));
                    }
                }
            }
        }
    }

    console.log_info("测试通过，打包完成！");
    
    index_file.save(&apppath.index_file);

    // // 生成上传脚本
    // let context = TemplateContext {
    //     upload_files: vec![version_file.strip_prefix(&ctx.working_dir).unwrap().to_str().unwrap().to_owned()],
    //     delete_files: Vec::new(),
    // };

    // generate_upload_script(context, ctx, &version_label);

    0
}

#[cfg(test)]
mod tests {
    use super::block_on_future;

    #[test]
    fn block_on_future_works_outside_runtime() {
        let result = block_on_future(async { 7 });
        assert_eq!(result, 7);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn block_on_future_works_inside_runtime() {
        let result = block_on_future(async { 11 });
        assert_eq!(result, 11);
    }
}
