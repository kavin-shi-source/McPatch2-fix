export function getManualChunkName(id) {
  if (!id.includes('/node_modules/')) {
    return undefined
  }

  if (
    id.includes('/node_modules/lucide-react/')
  ) {
    return 'vendor-icons'
  }

  if (
    id.includes('/node_modules/react/') ||
    id.includes('/node_modules/react-dom/') ||
    id.includes('/node_modules/react-router/') ||
    id.includes('/node_modules/react-router-dom/') ||
    id.includes('/node_modules/scheduler/') ||
    id.includes('/node_modules/antd/') ||
    id.includes('/node_modules/@ant-design/') ||
    id.includes('/node_modules/@ctrl/tinycolor/') ||
    id.includes('/node_modules/rc-field-form/') ||
    id.includes('/node_modules/rc-select/') ||
    id.includes('/node_modules/rc-input/') ||
    id.includes('/node_modules/rc-textarea/') ||
    id.includes('/node_modules/rc-picker/') ||
    id.includes('/node_modules/rc-dialog/') ||
    id.includes('/node_modules/rc-motion/') ||
    id.includes('/node_modules/rc-notification/') ||
    id.includes('/node_modules/rc-trigger/') ||
    id.includes('/node_modules/rc-tooltip/') ||
    id.includes('/node_modules/rc-upload/') ||
    id.includes('/node_modules/rc-util/') ||
    id.includes('/node_modules/rc-resize-observer/') ||
    id.includes('/node_modules/async-validator/')
  ) {
    return 'vendor-react'
  }

  if (
    id.includes('/node_modules/@reduxjs/') ||
    id.includes('/node_modules/react-redux/') ||
    id.includes('/node_modules/redux/') ||
    id.includes('/node_modules/reselect/') ||
    id.includes('/node_modules/immer/')
  ) {
    return 'vendor-redux'
  }

  return undefined
}
