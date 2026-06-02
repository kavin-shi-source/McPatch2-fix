import React from 'react';
import {Button, Card, Form, Input, message} from "antd";
import {userChangePasswordRequest, userChangeUsernameRequest} from "@/api/user.js";
import {useNavigate} from "react-router-dom";
import {useDispatch} from "react-redux";
import {clearToken} from "@/store/modules/userStore.js";

const Index = () => {

  const navigate = useNavigate();
  const dispatch = useDispatch();
  const [messageApi, contextHolder] = message.useMessage();
  const [passwordLoading, setPasswordLoading] = React.useState(false);
  const [usernameLoading, setUsernameLoading] = React.useState(false);

  const submitChangeUsername = async (values) => {
    setUsernameLoading(true)
    const {code, msg, data} = await userChangeUsernameRequest(values.newUsername);
    if (code === 1) {
      dispatch(clearToken())
      navigate('/login?type=changeUsername');
    } else {
      messageApi.error(msg)
    }
    setUsernameLoading(false)
  }

  const submitChangePassword = async (values) => {
    const passwordRegex = /^(?=.*[a-zA-Z])(?=.*\d).{8,}$/;
    if (!passwordRegex.test(values.newPassword)) {
      messageApi.warning('密码至少需要8位，且包含字母和数字!');
      return;
    }
    if (values.newPassword !== values.confirmPassword) {
      messageApi.warning('两次输入的密码不一致!');
      return;
    }
    setPasswordLoading(true)
    const {code, msg, data} = await userChangePasswordRequest(values.oldPassword, values.newPassword);
    if (code === 1) {
      dispatch(clearToken())
      navigate('/login?type=changePassword');
    } else {
      messageApi.error(msg)
    }
    setPasswordLoading(false)
  }

  return (
    <>
      {contextHolder}
      <div className="p-10 min-h-screen">
        <Card title="修改用户名" className="w-80 shadow-[0_4px_6px_rgba(0,0,0,0.1)] ">
          <Form
            layout="vertical"
            initialValues={{layout: 'vertical'}}
            onFinish={submitChangeUsername}>
            <Form.Item label="新用户名" name="newUsername" rules={[{required: true, message: '请输入新用户名!'}]}>
              <Input placeholder="请输入想要设置的新用户名."/>
            </Form.Item>
            <Form.Item>
              <Button type="primary" htmlType="submit" className="w-full" loading={usernameLoading}>保存</Button>
            </Form.Item>
          </Form>
        </Card>
        <Card title="修改密码" className="w-80 shadow-[0_4px_6px_rgba(0,0,0,0.1)] mt-5">
          <Form
            layout="vertical"
            initialValues={{layout: 'vertical'}}
            onFinish={submitChangePassword}>
            <Form.Item label="旧密码" name="oldPassword" rules={[{required: true, message: '请输入旧密码!'}]}>
              <Input placeholder="请输入旧密码."/>
            </Form.Item>
            <Form.Item label="新密码" name="newPassword" rules={[{required: true, message: '请输入新密码!'}]}>
              <Input placeholder="请输入想要设置的新密码."/>
            </Form.Item>
            <Form.Item label="确认新密码" name="confirmPassword" rules={[{required: true, message: '请确认新密码!'}]}>
              <Input placeholder="请再次输入新密码."/>
            </Form.Item>
            <Form.Item>
              <Button type="primary" htmlType="submit" className="w-full" loading={passwordLoading}>保存</Button>
            </Form.Item>
          </Form>
        </Card>
      </div>
    </>
  );
};

export default Index;
