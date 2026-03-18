# UI界面要放在<Oheader></Oheader>里面，而不是App.vue中，因为我做了自定义边框，所以要放在<Oheader></Oheader>里面。
# 弹窗/全局消息要放在<Oheader></Oheader>的<template #Modal></template>里面。
# 界面图标可以使用 lucide 和 phosphor-icons 图标库，可以混合使用。
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
