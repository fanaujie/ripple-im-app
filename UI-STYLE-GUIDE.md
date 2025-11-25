# Ripple IM App - UI 风格指南

## 目录
1. [技术栈](#技术栈)
2. [设计系统](#设计系统)
3. [组件设计模式](#组件设计模式)
4. [布局系统](#布局系统)
5. [交互规范](#交互规范)
6. [资源管理](#资源管理)

---

## 技术栈

### 核心技术
- **UI框架**: Vue 3.5.13 + TypeScript 5.6.2
- **样式方案**: Tailwind CSS v4.1.11
- **构建工具**: Vite 6.0.3
- **桌面框架**: Tauri 2.x
- **CSS处理**: PostCSS + Autoprefixer

### 样式架构
- 主样式文件: `src/assets/style.css`
- 使用Tailwind v4 `@theme` 指令定义设计token
- 无第三方组件库依赖（完全自定义）

---

## 设计系统

### 色彩系统

#### 主色调
```css
--color-primary: #3b82f6;      /* 蓝色 - 品牌主色 */
--color-secondary: #64748b;    /* 灰蓝 - 辅助色 */
--color-accent: #10b981;       /* 绿色 - 强调色 */
```

#### 背景色
```css
--color-background: #f1f5f9;   /* 应用背景 - 浅灰 */
--color-surface: #ffffff;      /* 卡片表面 - 白色 */
--color-sidebar: #1e293b;      /* 侧边栏 - 深灰 */
--color-sidebar-hover: #334155; /* 侧边栏悬停 */
```

#### 文本色
```css
--color-text: #1e293b;                    /* 主要文本 */
--color-text-secondary: #64748b;          /* 次要文本 */
--color-text-sidebar: #ffffff;            /* 侧边栏文本 */
--color-text-sidebar-secondary: #cbd5e1;  /* 侧边栏次要文本 */
```

#### 边框色
```css
--color-border: #e2e8f0;           /* 通用边框 */
--color-border-sidebar: #374151;   /* 侧边栏边框 */
```

#### 语义色（Tailwind类名）
```css
成功: text-green-600, bg-green-50
错误: text-red-600, bg-red-50, border-red-200
警告: text-yellow-600, bg-yellow-50
信息: text-blue-600, bg-blue-50
```

### 间距系统

基于 Tailwind 的 4px 单位系统：

```
p-2   = 8px
p-3   = 12px
p-4   = 16px
p-6   = 24px
p-8   = 32px
p-12  = 48px
```

#### 常用间距模式
- **卡片内边距**: `p-4` (16px) 或 `p-6` (24px)
- **页面内边距**: `px-8 py-6` (水平32px，垂直24px)
- **按钮内边距**: `px-4 py-2` (水平16px，垂直8px)
- **组件间距**: `gap-2` (8px), `gap-3` (12px), `gap-4` (16px)

### 字体系统

#### 字体族
```css
--font-family-sans: Inter, sans-serif;
```

从 Google Fonts 加载，支持字重: 300, 400, 500, 600, 700

#### 字体大小
```
text-xs    = 12px   /* 辅助文本 */
text-sm    = 14px   /* 次要信息、按钮 */
text-base  = 16px   /* 正文内容 */
text-lg    = 18px   /* 子标题 */
text-xl    = 20px   /* 页面标题 */
text-2xl   = 24px   /* 主标题 */
text-3xl   = 30px   /* 大标题 */
```

#### 字重
```
font-medium   = 500   /* 一般强调 */
font-semibold = 600   /* 标题、重要信息 */
```

### 圆角系统

```
rounded-lg    = 8px     /* 按钮、输入框、小卡片 */
rounded-xl    = 12px    /* 卡片、模态框 */
rounded-2xl   = 16px    /* 对话框 */
rounded-full  = 50%     /* 头像、徽章、指示器 */
```

### 阴影系统

```
shadow-sm   /* 轻微阴影 - 头部栏 */
shadow-md   /* 中等阴影 - 卡片悬停 */
shadow-lg   /* 较大阴影 - 下拉菜单、对话框 */
```

### 布局尺寸

```css
--width-sidebar: 256px;   /* 侧边栏宽度 */
--height-header: 64px;    /* 头部高度 */
```

---

## 组件设计模式

### 按钮

#### 主要按钮
```vue
<button class="px-4 py-2 bg-primary text-white rounded-lg
               hover:bg-primary/90 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed">
  按钮文本
</button>
```

#### 次要按钮
```vue
<button class="px-4 py-2 border border-gray-300 rounded-lg
               hover:bg-gray-50 transition-colors">
  按钮文本
</button>
```

#### 危险按钮
```vue
<button class="px-4 py-2 bg-red-600 text-white rounded-lg
               hover:bg-red-700 transition-colors">
  删除
</button>
```

#### 图标按钮
```vue
<button class="p-2 rounded-lg hover:bg-gray-100 transition-colors">
  <HeroIcon name="edit-2" className="w-5 h-5" />
</button>
```

### 输入框

#### 标准输入框
```vue
<input
  type="text"
  class="w-full px-4 py-3 bg-white border border-gray-200 rounded-lg
         focus:outline-none focus:ring-2 focus:ring-blue-500
         focus:border-transparent"
  placeholder="请输入..."
/>
```

#### 搜索框
```vue
<div class="relative">
  <HeroIcon name="search" className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
  <input
    class="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg
           focus:outline-none focus:ring-2 focus:ring-primary/20"
    placeholder="搜索..."
  />
</div>
```

#### 错误状态
```vue
<input
  class="w-full px-4 py-3 border border-red-200 rounded-lg
         bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-500"
/>
<p class="mt-1 text-sm text-red-600">错误提示信息</p>
```

### 卡片

#### 基础卡片
```vue
<div class="bg-white rounded-lg border border-gray-200 p-4
            hover:shadow-md transition-shadow">
  <!-- 内容 -->
</div>
```

#### 交互卡片
```vue
<div class="bg-white rounded-lg border border-gray-200 p-4
            hover:shadow-md hover:border-primary/50
            transition-all cursor-pointer">
  <!-- 内容 -->
</div>
```

### 头像

#### 圆形头像
```vue
<img
  :src="avatarUrl"
  @error="onImageError"
  class="w-12 h-12 rounded-full object-cover"
  alt="用户头像"
/>
```

#### 带状态指示器的头像
```vue
<div class="relative">
  <img src="avatar.jpg" class="w-12 h-12 rounded-full" />
  <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-500
              border-2 border-white rounded-full"></div>
</div>
```

### 徽章

```vue
<!-- 成功徽章 -->
<span class="px-3 py-1 bg-green-50 text-green-600 rounded-full text-sm font-medium">
  已添加
</span>

<!-- 警告徽章 -->
<span class="px-3 py-1 bg-yellow-50 text-yellow-600 rounded-full text-sm">
  待确认
</span>

<!-- 信息徽章 -->
<span class="px-3 py-1 bg-blue-50 text-blue-600 rounded-full text-sm">
  Coming Soon
</span>
```

### 下拉菜单

```vue
<div class="absolute right-0 mt-2 w-48 bg-white border border-gray-200
            rounded-lg shadow-lg py-2 z-10">
  <button class="w-full px-4 py-2 text-left hover:bg-gray-50
                 flex items-center gap-3 transition-colors">
    <HeroIcon name="edit-2" className="w-4 h-4 text-gray-400" />
    <span class="text-gray-700">编辑</span>
  </button>
  <button class="w-full px-4 py-2 text-left hover:bg-gray-50
                 flex items-center gap-3 text-red-600">
    <HeroIcon name="trash" className="w-4 h-4" />
    <span>删除</span>
  </button>
</div>
```

### 模态框/对话框

```vue
<!-- 遮罩层 -->
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
  <!-- 对话框容器 -->
  <div class="bg-white rounded-2xl border border-border max-w-md w-full mx-4">
    <!-- 头部 -->
    <div class="px-6 py-4 border-b border-border">
      <h3 class="text-lg font-semibold">对话框标题</h3>
    </div>

    <!-- 内容 -->
    <div class="p-6">
      <p class="text-gray-600">对话框内容</p>
    </div>

    <!-- 操作按钮 -->
    <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
      <button class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
        取消
      </button>
      <button class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90">
        确认
      </button>
    </div>
  </div>
</div>
```

### 标签页

```vue
<div class="flex gap-6 border-b border-gray-200">
  <button
    :class="[
      'pb-3 font-medium transition-colors',
      activeTab === 'tab1'
        ? 'text-blue-500 border-b-2 border-blue-500'
        : 'text-gray-600 hover:text-gray-900'
    ]">
    标签1
  </button>
  <button
    :class="[
      'pb-3 font-medium transition-colors',
      activeTab === 'tab2'
        ? 'text-blue-500 border-b-2 border-blue-500'
        : 'text-gray-600 hover:text-gray-900'
    ]">
    标签2
  </button>
</div>
```

### 空状态

```vue
<div class="flex items-center justify-center h-full">
  <div class="text-center max-w-sm">
    <!-- 图标 -->
    <div class="w-16 h-16 mx-auto bg-primary/10 rounded-full
                flex items-center justify-center mb-4">
      <HeroIcon name="inbox" className="w-8 h-8 text-primary" />
    </div>

    <!-- 标题 -->
    <h3 class="text-lg font-semibold text-gray-900 mb-2">
      暂无内容
    </h3>

    <!-- 描述 -->
    <p class="text-gray-500 mb-6">
      这里还没有任何内容
    </p>

    <!-- 操作按钮 -->
    <button class="px-6 py-2 bg-primary text-white rounded-lg
                   hover:bg-primary/90 transition-colors">
      开始使用
    </button>
  </div>
</div>
```

### 加载状态

```vue
<!-- 加载遮罩 -->
<div class="fixed inset-0 bg-white/80 flex items-center justify-center z-50">
  <div class="text-center">
    <div class="w-12 h-12 border-4 border-primary/20 border-t-primary
                rounded-full animate-spin mx-auto mb-4"></div>
    <p class="text-gray-600">加载中...</p>
  </div>
</div>

<!-- 内联加载 -->
<button disabled class="px-4 py-2 bg-primary/50 text-white rounded-lg
                       flex items-center gap-2 cursor-not-allowed">
  <div class="w-4 h-4 border-2 border-white/30 border-t-white
              rounded-full animate-spin"></div>
  <span>加载中...</span>
</button>
```

---

## 布局系统

### 主布局架构

```vue
<div class="h-screen flex bg-background">
  <!-- 侧边栏 -->
  <NavigationSidebar />  <!-- 256px固定宽度 -->

  <!-- 主内容区 -->
  <main class="flex-1 h-full overflow-hidden">
    <router-view />
  </main>
</div>
```

### 页面布局模板

#### 标准页面结构
```vue
<div class="flex flex-col h-full">
  <!-- 头部 -->
  <div class="bg-white border-b border-gray-200 px-8 py-6 shadow-sm">
    <h1 class="text-3xl font-semibold text-gray-900">页面标题</h1>
  </div>

  <!-- 内容区 -->
  <div class="flex-1 bg-gray-50 px-8 py-6 overflow-auto">
    <!-- 页面内容 -->
  </div>
</div>
```

#### 带侧边栏的页面
```vue
<div class="flex h-full">
  <!-- 左侧列表 -->
  <div class="w-80 border-r border-gray-200 bg-white overflow-auto">
    <!-- 列表内容 -->
  </div>

  <!-- 右侧详情 -->
  <div class="flex-1 overflow-auto">
    <!-- 详情内容 -->
  </div>
</div>
```

### 容器宽度

```vue
<div class="max-w-4xl mx-auto p-8">  <!-- 设置页面 - 最大1024px -->
<div class="max-w-2xl mx-auto">      <!-- 表单页面 - 最大672px -->
<div class="max-w-md mx-auto">       <!-- 登录/对话框 - 最大448px -->
```

### 网格布局

#### 响应式网格
```vue
<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  <!-- 网格项 -->
</div>
```

#### 固定列网格
```vue
<div class="grid grid-cols-3 gap-4">
  <!-- 3列布局 -->
</div>
```

### Flexbox布局

#### 水平居中
```vue
<div class="flex items-center justify-center">
  <!-- 居中内容 -->
</div>
```

#### 两端对齐
```vue
<div class="flex items-center justify-between">
  <div>左侧内容</div>
  <div>右侧内容</div>
</div>
```

#### 垂直堆叠
```vue
<div class="flex flex-col gap-4">
  <div>项目1</div>
  <div>项目2</div>
</div>
```

---

## 交互规范

### 状态样式

#### 悬停（Hover）
```css
hover:bg-gray-50          /* 背景变化 */
hover:text-gray-700       /* 文本颜色 */
hover:shadow-md           /* 阴影增强 */
hover:border-primary      /* 边框高亮 */
hover:scale-105           /* 轻微放大 */
```

#### 焦点（Focus）
```css
focus:outline-none
focus:ring-2 focus:ring-primary/20
focus:border-primary
```

#### 激活（Active）
```css
active:scale-95           /* 按下缩小 */
active:bg-primary/80      /* 颜色加深 */
```

#### 禁用（Disabled）
```css
disabled:opacity-50
disabled:cursor-not-allowed
disabled:pointer-events-none
```

### 过渡动画

#### 标准过渡
```css
transition-all duration-200        /* 综合过渡 */
transition-colors duration-200     /* 颜色过渡 */
transition-shadow duration-300     /* 阴影过渡 */
transition-transform duration-200  /* 变换过渡 */
```

#### 动画效果
```css
animate-spin              /* 旋转动画 - 加载指示器 */
animate-pulse             /* 脉冲动画 - 提示 */
animate-bounce            /* 弹跳动画 - 提示 */
```

### 交互反馈原则

1. **即时反馈**: 所有可点击元素必须有 hover 状态
2. **视觉层次**: 使用阴影和颜色区分元素重要性
3. **平滑过渡**: 状态变化使用 200-300ms 过渡
4. **明确状态**: 激活、禁用、加载状态要明显区分
5. **防误操作**: 危险操作需要二次确认

---

## 资源管理

### 图标系统

使用自定义 SVG 图标组件：`HeroIcon.vue`

#### 可用图标
```
chat-bubble-left-right    聊天
users                     用户
cog-6-tooth               设置
arrow-left-on-rectangle   登出
ellipsis-horizontal       更多（横向）
more-vertical             更多（纵向）
user-plus                 添加好友
edit-2                    编辑
message-circle            消息
eye / eye-off             显示/隐藏
search                    搜索
trash                     删除
inbox                     收件箱
```

#### 使用方式
```vue
<HeroIcon name="users" className="w-5 h-5 text-gray-400" />
```

#### 图标尺寸规范
```
w-4 h-4   = 16px   /* 小图标 - 内联文本 */
w-5 h-5   = 20px   /* 标准图标 - 按钮、导航 */
w-6 h-6   = 24px   /* 大图标 - 头部、重要操作 */
w-8 h-8   = 32px   /* 特大图标 - 空状态、展示 */
```

### 图片处理

#### 头像加载
```typescript
// 默认头像
const defaultAvatarUrl = new URL('../assets/default-avatar.svg', import.meta.url).href;

// 头像URL处理
const getAvatarUrl = (avatarPath?: string) => {
  if (!avatarPath) return defaultAvatarUrl;
  if (avatarPath.startsWith('http://') || avatarPath.startsWith('https://')) {
    return avatarPath;  // 远程图片
  }
  return `asset://localhost/${avatarPath}`;  // Tauri本地资源
};

// 错误回退
const onImageError = (event: Event) => {
  (event.target as HTMLImageElement).src = defaultAvatarUrl;
};
```

#### 图片上传验证
```typescript
// 验证文件类型
const validateImageType = (file: File): boolean => {
  return file.type.startsWith('image/');
};

// 验证文件大小
const validateImageSize = (file: File, maxSizeMB: number = 5): boolean => {
  return file.size <= maxSizeMB * 1024 * 1024;
};

// 验证图片尺寸
const validateImageDimensions = async (
  file: File,
  minWidth: number = 460,
  minHeight: number = 460
): Promise<boolean> => {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      resolve(img.width >= minWidth && img.height >= minHeight);
    };
    img.onerror = () => resolve(false);
    img.src = URL.createObjectURL(file);
  });
};
```

### 字体加载

```css
/* 从 Google Fonts 加载 Inter 字体 */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap');
```

---

## 最佳实践

### 代码规范

#### 1. 组件结构
```vue
<template>
  <!-- 模板内容 -->
</template>

<script setup lang="ts">
// 导入
import { ref, computed, onMounted } from 'vue';
import type { User } from '@/types';

// Props
interface Props {
  user: User;
  isActive?: boolean;
}
const props = withDefaults(defineProps<Props>(), {
  isActive: false
});

// Emits
const emit = defineEmits<{
  update: [user: User];
  delete: [id: string];
}>();

// 响应式状态
const isLoading = ref(false);

// 计算属性
const displayName = computed(() => {
  return props.user.nickname || props.user.username;
});

// 方法
const handleClick = () => {
  emit('update', props.user);
};

// 生命周期
onMounted(() => {
  // 初始化逻辑
});
</script>

<style scoped>
/* 仅在需要特殊样式时使用 */
</style>
```

#### 2. Tailwind类名顺序
```vue
<!-- 推荐顺序: 布局 > 尺寸 > 间距 > 字体 > 颜色 > 边框 > 效果 > 过渡 -->
<div class="
  flex items-center justify-between
  w-full h-12
  px-4 py-2 gap-3
  text-base font-medium
  text-gray-900 bg-white
  border border-gray-200 rounded-lg
  shadow-sm
  hover:bg-gray-50 transition-colors
">
```

#### 3. 动态类名
```vue
<!-- 使用数组语法 -->
<button :class="[
  'px-4 py-2 rounded-lg transition-colors',
  isActive
    ? 'bg-primary text-white'
    : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
]">

<!-- 使用对象语法 -->
<div :class="{
  'bg-primary text-white': isActive,
  'bg-gray-100 text-gray-700': !isActive,
  'opacity-50 cursor-not-allowed': isDisabled
}">
```

### 性能优化

#### 1. 图片懒加载
```vue
<img
  :src="avatarUrl"
  loading="lazy"
  class="w-12 h-12 rounded-full"
/>
```

#### 2. 组件懒加载
```typescript
// router/index.ts
const routes = [
  {
    path: '/settings',
    component: () => import('@/views/SettingsView.vue')
  }
];
```

#### 3. 使用 KeepAlive
```vue
<keep-alive :include="['ChatView', 'PeopleView', 'SettingsView']">
  <component :is="currentView" />
</keep-alive>
```

### 可访问性

#### 1. 语义化HTML
```vue
<button>按钮</button>                    <!-- 而非 <div @click> -->
<input type="text" />                    <!-- 正确的输入类型 -->
<nav>导航</nav>                          <!-- 语义化标签 -->
```

#### 2. ARIA属性
```vue
<button
  aria-label="删除用户"
  aria-pressed="false"
>
  <HeroIcon name="trash" />
</button>

<input
  type="search"
  aria-label="搜索用户"
  placeholder="搜索..."
/>
```

#### 3. 键盘导航
```vue
<button
  @click="handleClick"
  @keydown.enter="handleClick"
  @keydown.space.prevent="handleClick"
  tabindex="0"
>
```

---

## 扩展方向

### 短期优化
1. 统一 Login 页面样式（迁移到 Tailwind）
2. 提取可复用组件（Button、Input、Card、Avatar等）
3. 补充更多图标
4. 添加更多语义化颜色变量（success、error、warning等）

### 中期规划
1. 实现深色主题切换
2. 添加响应式设计支持
3. 建立组件库文档（Storybook）
4. 添加动画库（更丰富的过渡效果）

### 长期规划
1. 设计系统国际化
2. 主题自定义功能
3. 组件单元测试
4. 性能监控与优化

---

## 文件索引

### 核心样式文件
- `src/assets/style.css` - 主样式文件
- `postcss.config.js` - PostCSS配置

### 组件文件
- `src/components/navigation/NavigationSidebar.vue` - 导航侧边栏
- `src/components/shared/HeroIcon.vue` - 图标组件
- `src/components/views/EmptyView.vue` - 空状态组件

### 视图文件
- `src/views/Login.vue` - 登录页
- `src/views/Home.vue` - 主框架
- `src/views/ChatView.vue` - 聊天视图
- `src/views/PeopleView.vue` - 人员管理
- `src/views/SettingsView.vue` - 设置页面

### 资源文件
- `src/assets/default-avatar.svg` - 默认头像
- `src/assets/vue.svg` - Vue标志

---

## 版本信息

- **文档版本**: 1.0.0
- **创建日期**: 2025-11-05
- **适用版本**: Ripple IM App v0.0.0
- **Tailwind版本**: v4.1.11
- **Vue版本**: v3.5.13

---

## 附录

### Tailwind配置参考

当前项目使用 Tailwind CSS v4，配置在 `src/assets/style.css` 中通过 `@theme` 指令定义。

如需迁移到独立配置文件，参考结构：

```javascript
// tailwind.config.js (参考)
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts}'],
  theme: {
    extend: {
      colors: {
        primary: '#3b82f6',
        secondary: '#64748b',
        accent: '#10b981',
        // ... 其他颜色
      },
      spacing: {
        'sidebar': '256px',
        'header': '64px',
      }
    }
  }
}
```

### 设计token完整列表

参考 `src/assets/style.css` 中的 `@theme` 块获取完整的设计token列表。

### 相关资源

- [Tailwind CSS 文档](https://tailwindcss.com/docs)
- [Vue 3 文档](https://vuejs.org/)
- [Tauri 文档](https://tauri.app/)
- [Heroicons](https://heroicons.com/) - 图标设计参考

---

**文档维护**: 请在UI系统发生重大变更时更新此文档。
