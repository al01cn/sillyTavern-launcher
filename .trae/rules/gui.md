# UI界面要放在<Oheader></Oheader>里面，而不是App.vue中，因为我做了自定义边框，所以要放在<Oheader></Oheader>里面。
# 弹窗/全局消息要放在<Oheader></Oheader>的<template #Modal></template>里面。
# 界面图标可以使用 lucide 和 phosphor-icons 图标库，可以混合使用。
# 界面UI要使用已经安装的UI库，还有已有的UI样式，不能自己写样式，也不要改原来的样式。
# 界面风格要保持一致，不能有不同的风格。
# 软件因为要在全平台（Windows、macOS、Linux）上运行，所以写功能的时候要确保在所有平台上都能正常运行，而且要设置回退机制，避免因为平台差异导致的问题。
# 接口请求要从rust后端发起请求，不能直接从前端发起请求，前端只接受后端返回的数据，为了避免跨域问题。
# 所有与系统交互的操作，都要通过rust后端编写函数/接口给前端按钮调用。

# 软件目录结构如下
data 
├── config.json // 配置文件
├── node // node 核心目录
├── sillytavern // sillytavern 核心目录
├── plugins // sillytavern 待安装插件目录
└── logs // 日志目录
    └── app.log // 应用日志文件
