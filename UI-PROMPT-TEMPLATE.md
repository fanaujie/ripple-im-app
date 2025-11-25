# Ripple IM App - 新UI功能开发 Prompt 模板

## 📋 使用说明

1. 复制本模板的【需求描述部分】（下方标注的区域）
2. 填写你的具体需求
3. 将完整的prompt提供给AI助手
4. AI将根据项目现有架构和风格生成代码

---

## 🎯 【需求描述部分 - 请填写】

### 1. 功能概述
<!-- 简要描述这个UI的目的和主要功能 -->
**功能名称**：[例如：用户资料编辑页面]

**功能描述**：
[描述这个UI要实现什么功能，解决什么问题]

### 2. UI类型
<!-- 选择一个或多个 -->
- [ ] 完整页面 (View)
- [ ] 可复用组件 (Component)
- [ ] 模态框/对话框 (Modal/Dialog)
- [ ] 侧边栏/抽屉 (Sidebar/Drawer)
- [ ] 其他：___________

### 3. 页面结构
<!-- 描述页面的主要区域和布局 -->
**布局类型**：
- [ ] 标准页面布局（Header + Content）
- [ ] 左右分栏布局（List + Detail）
- [ ] 三栏布局
- [ ] 对话框/模态框
- [ ] 其他：___________

**主要区域**：
```
例如：
- 顶部：页面标题 + 操作按钮
- 内容区：
  - 左侧：筛选/分类列表
  - 右侧：详情展示区
```

### 4. 主要功能点
<!-- 列出这个UI需要实现的具体功能 -->
1. [功能点1，例如：显示用户列表]
2. [功能点2，例如：支持搜索和筛选]
3. [功能点3，例如：点击查看详情]
4. ...

### 5. 交互行为
<!-- 描述用户的主要操作流程和交互 -->
**用户操作流程**：
1. [步骤1]
2. [步骤2]
3. ...

**需要的交互组件**：
- [ ] 按钮（主要/次要/危险）
- [ ] 输入框（文本/搜索/数字等）
- [ ] 下拉菜单
- [ ] 标签页
- [ ] 复选框/单选框
- [ ] 开关
- [ ] 文件上传
- [ ] 日期/时间选择器
- [ ] 其他：___________

### 6. 数据需求
<!-- 描述需要展示和处理的数据 -->
**数据结构**（如果已知）：
```typescript
// 例如：
interface UserProfile {
  id: string;
  name: string;
  avatar?: string;
  // ...
}
```

**数据来源**：
- [ ] 需要创建新的类型定义（types/）
- [ ] 使用现有类型：___________
- [ ] 需要创建新的Composable
- [ ] 使用现有Composable：___________
- [ ] 需要后端API对接
- [ ] 使用Mock数据

### 7. 状态管理
<!-- 描述需要管理的状态 -->
**需要的状态**：
- [例如：加载状态、搜索关键词、选中项等]

### 8. 特殊要求
<!-- 其他特殊需求或约束 -->
- [ ] 需要响应式设计
- [ ] 需要动画效果（描述：___________）
- [ ] 需要权限控制
- [ ] 需要表单验证（规则：___________）
- [ ] 需要错误处理
- [ ] 需要空状态展示
- [ ] 需要加载状态展示
- [ ] 其他：___________

### 9. 参考示例
<!-- 可选：如果项目中有类似的页面或组件，可以引用 -->
**参考现有页面/组件**：
- [例如：类似于 PeopleView.vue 的布局]
- [例如：使用与 SettingsView.vue 相同的卡片风格]

---

## 🏗️ 【项目技术规范 - 自动填充，无需修改】

### 技术栈
- **UI框架**：Vue 3.5.13 + TypeScript 5.6.2
- **样式方案**：Tailwind CSS v4.1.11
- **构建工具**：Vite 6.0.3
- **桌面框架**：Tauri 2.x

### 项目结构
```
src/
├── views/              # 页面视图
├── components/         # 可复用组件
│   ├── navigation/    # 导航组件
│   ├── shared/        # 共享组件
│   └── views/         # 视图专用组件
├── composables/        # 组合式函数（业务逻辑）
├── types/              # TypeScript类型定义
├── router/             # 路由配置
└── assets/             # 静态资源

命名规范：
- Views: XxxView.vue (例如：UserProfileView.vue)
- Components: XxxComponent.vue 或直接功能名 (例如：UserCard.vue)
- Composables: useXxx.ts (例如：useUserProfile.ts)
- Types: xxx.ts (例如：user-profile.ts)
```

### UI设计系统

#### 色彩系统
```css
/* 主色调 */
--color-primary: #3b82f6;      /* 蓝色 - 品牌主色 */
--color-secondary: #64748b;    /* 灰蓝 - 辅助色 */
--color-accent: #10b981;       /* 绿色 - 强调色 */

/* 背景色 */
--color-background: #f1f5f9;   /* 应用背景 */
--color-surface: #ffffff;      /* 卡片表面 */
--color-sidebar: #1e293b;      /* 侧边栏 */

/* 文本色 */
--color-text: #1e293b;         /* 主要文本 */
--color-text-secondary: #64748b; /* 次要文本 */

/* 边框色 */
--color-border: #e2e8f0;
```

**Tailwind语义色类名**：
- 成功：`text-green-600`, `bg-green-50`
- 错误：`text-red-600`, `bg-red-50`, `border-red-200`
- 警告：`text-yellow-600`, `bg-yellow-50`
- 信息：`text-blue-600`, `bg-blue-50`

#### 间距系统
```
p-2  = 8px   | p-3  = 12px  | p-4  = 16px
p-6  = 24px  | p-8  = 32px  | p-12 = 48px

常用模式：
- 卡片内边距: p-4 或 p-6
- 页面内边距: px-8 py-6
- 按钮内边距: px-4 py-2
- 组件间距: gap-2, gap-3, gap-4
```

#### 字体系统
```
text-xs   = 12px  /* 辅助文本 */
text-sm   = 14px  /* 次要信息、按钮 */
text-base = 16px  /* 正文内容 */
text-lg   = 18px  /* 子标题 */
text-xl   = 20px  /* 页面标题 */
text-2xl  = 24px  /* 主标题 */
text-3xl  = 30px  /* 大标题 */

font-medium   = 500  /* 一般强调 */
font-semibold = 600  /* 标题、重要信息 */
```

#### 圆角系统
```
rounded-lg   = 8px   /* 按钮、输入框、小卡片 */
rounded-xl   = 12px  /* 卡片、模态框 */
rounded-2xl  = 16px  /* 对话框 */
rounded-full = 50%   /* 头像、徽章 */
```

### 组件设计模式

#### 1. 按钮
```vue
<!-- 主要按钮 -->
<button class="px-4 py-2 bg-primary text-white rounded-lg
               hover:bg-primary/90 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed">
  按钮文本
</button>

<!-- 次要按钮 -->
<button class="px-4 py-2 border border-gray-300 rounded-lg
               hover:bg-gray-50 transition-colors">
  按钮文本
</button>

<!-- 危险按钮 -->
<button class="px-4 py-2 bg-red-600 text-white rounded-lg
               hover:bg-red-700 transition-colors">
  删除
</button>

<!-- 图标按钮 -->
<button class="p-2 rounded-lg hover:bg-gray-100 transition-colors">
  <HeroIcon name="edit-2" className="w-5 h-5" />
</button>
```

#### 2. 输入框
```vue
<!-- 标准输入框 -->
<input
  type="text"
  class="w-full px-4 py-3 bg-white border border-gray-200 rounded-lg
         focus:outline-none focus:ring-2 focus:ring-blue-500
         focus:border-transparent"
  placeholder="请输入..."
/>

<!-- 搜索框 -->
<div class="relative">
  <HeroIcon name="magnifying-glass" className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
  <input
    class="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg
           focus:outline-none focus:ring-2 focus:ring-primary/20"
    placeholder="搜索..."
  />
</div>

<!-- 错误状态 -->
<input class="w-full px-4 py-3 border border-red-200 rounded-lg
              bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-500" />
<p class="mt-1 text-sm text-red-600">错误提示信息</p>
```

#### 3. 卡片
```vue
<!-- 基础卡片 -->
<div class="bg-white rounded-lg border border-gray-200 p-4
            hover:shadow-md transition-shadow">
  <!-- 内容 -->
</div>

<!-- 交互卡片 -->
<div class="bg-white rounded-lg border border-gray-200 p-4
            hover:shadow-md hover:border-primary/50
            transition-all cursor-pointer">
  <!-- 内容 -->
</div>
```

#### 4. 模态框
```vue
<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
  <div class="bg-white rounded-2xl border border-border max-w-md w-full mx-4">
    <!-- 头部 -->
    <div class="px-6 py-4 border-b border-border">
      <h3 class="text-lg font-semibold">标题</h3>
    </div>
    <!-- 内容 -->
    <div class="p-6">
      <p class="text-gray-600">内容</p>
    </div>
    <!-- 操作 -->
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

#### 5. 标签页
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
</div>
```

#### 6. 空状态
```vue
<div class="flex items-center justify-center h-full">
  <div class="text-center max-w-sm">
    <div class="w-16 h-16 mx-auto bg-primary/10 rounded-full
                flex items-center justify-center mb-4">
      <HeroIcon name="inbox" className="w-8 h-8 text-primary" />
    </div>
    <h3 class="text-lg font-semibold text-gray-900 mb-2">暂无内容</h3>
    <p class="text-gray-500 mb-6">这里还没有任何内容</p>
    <button class="px-6 py-2 bg-primary text-white rounded-lg
                   hover:bg-primary/90 transition-colors">
      开始使用
    </button>
  </div>
</div>
```

#### 7. 加载状态
```vue
<!-- 加载指示器 -->
<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>

<!-- 加载按钮 -->
<button disabled class="px-4 py-2 bg-primary/50 text-white rounded-lg
                       flex items-center gap-2 cursor-not-allowed">
  <div class="w-4 h-4 border-2 border-white/30 border-t-white
              rounded-full animate-spin"></div>
  <span>加载中...</span>
</button>
```

### Vue 3 代码规范

#### 组件结构模板
```vue
<template>
  <!-- 模板内容 -->
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { YourType } from '@/types/your-type';
import HeroIcon from '@/components/shared/HeroIcon.vue';

// 定义组件名（用于KeepAlive）
defineOptions({
  name: 'YourComponentName'
});

// Props
interface Props {
  data: YourType;
  optional?: boolean;
}
const props = withDefaults(defineProps<Props>(), {
  optional: false
});

// Emits
const emit = defineEmits<{
  update: [data: YourType];
  delete: [id: string];
}>();

// 响应式状态
const isLoading = ref(false);
const items = ref<YourType[]>([]);

// 计算属性
const displayValue = computed(() => {
  return props.data.value || 'default';
});

// 方法
const handleClick = () => {
  emit('update', props.data);
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

#### Tailwind 类名顺序
```
推荐顺序: 布局 > 尺寸 > 间距 > 字体 > 颜色 > 边框 > 效果 > 过渡

示例：
class="
  flex items-center justify-between
  w-full h-12
  px-4 py-2 gap-3
  text-base font-medium
  text-gray-900 bg-white
  border border-gray-200 rounded-lg
  shadow-sm
  hover:bg-gray-50 transition-colors
"
```

#### 可用图标（HeroIcon组件）
```
常用图标：
- magnifying-glass (搜索)
- chat-bubble-left-right (聊天)
- users (用户)
- cog-6-tooth (设置)
- user-plus (添加用户)
- edit-2 (编辑)
- trash (删除)
- eye / eye-off (显示/隐藏)
- message-circle (消息)
- more-vertical (更多菜单)
- arrow-left-on-rectangle (登出)
- inbox (收件箱/空状态)
- no-symbol (禁止/屏蔽)

使用方式：
<HeroIcon name="users" className="w-5 h-5 text-gray-400" />

图标尺寸：
w-4 h-4 = 16px  /* 小图标 */
w-5 h-5 = 20px  /* 标准图标 */
w-6 h-6 = 24px  /* 大图标 */
w-8 h-8 = 32px  /* 特大图标 */
```

### 布局模板

#### 标准页面布局
```vue
<template>
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
</template>
```

#### 左右分栏布局
```vue
<template>
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
</template>
```

---

## 📤 【输出要求】

请基于以上需求和规范，生成以下内容：

### 1. 文件结构
明确需要创建/修改哪些文件：
- [ ] Views（页面视图）
- [ ] Components（组件）
- [ ] Types（类型定义）
- [ ] Composables（组合式函数）
- [ ] Router（路由配置）

### 2. 完整代码
提供所有文件的完整代码，包括：
- Vue组件代码（template + script + 必要的style）
- TypeScript类型定义
- Composable函数（如需要）
- 路由配置（如需要）

### 3. 代码说明
简要说明：
- 主要功能实现思路
- 关键交互逻辑
- 数据流向
- 需要注意的事项

### 4. 集成指南
如何将生成的代码集成到项目中：
- 文件放置位置
- 需要的依赖导入
- 路由配置（如适用）
- 其他配置修改

---

## ✅ 质量检查清单

生成的代码应该满足：
- [ ] 遵循项目的Vue 3 Composition API风格
- [ ] 使用TypeScript类型定义
- [ ] 遵循Tailwind CSS设计系统
- [ ] 遵循UI组件设计模式
- [ ] 代码结构清晰，注释适当
- [ ] 包含加载、错误、空状态处理
- [ ] 响应式设计（如需要）
- [ ] 可访问性良好（语义化HTML、ARIA属性）

---

## 📝 示例：如何使用本模板

### 示例需求
假设你要创建一个"通知中心"页面，可以这样填写：

```markdown
### 1. 功能概述
**功能名称**：通知中心

**功能描述**：
展示系统通知和消息提醒，用户可以查看、标记已读、删除通知

### 2. UI类型
- [x] 完整页面 (View)

### 3. 页面结构
**布局类型**：
- [x] 标准页面布局（Header + Content）

**主要区域**：
- 顶部：页面标题 + "全部标记已读"按钮
- 内容区：通知列表（卡片形式）

### 4. 主要功能点
1. 显示所有通知列表
2. 区分已读/未读状态
3. 点击通知标记为已读
4. 删除单条通知
5. 全部标记为已读

### 5. 交互行为
**用户操作流程**：
1. 进入通知中心
2. 浏览通知列表
3. 点击通知查看详情（标记为已读）
4. 可单独删除或全部标记已读

**需要的交互组件**：
- [x] 按钮（主要/次要/危险）
- [x] 卡片列表

### 6. 数据需求
**数据结构**：
interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  isRead: boolean;
  timestamp: number;
}

**数据来源**：
- [x] 需要创建新的类型定义（types/notification.ts）
- [x] 需要创建新的Composable（useNotifications.ts）
- [x] 使用Mock数据

### 7. 状态管理
- 通知列表
- 加载状态
- 未读数量

### 8. 特殊要求
- [x] 需要空状态展示
- [x] 需要加载状态展示
- [x] 不同类型通知用不同颜色标识

### 9. 参考示例
类似于 PeopleView.vue 的列表布局，使用卡片展示每条通知
```

---

## 🔗 相关资源

- [详细UI风格指南](./UI-STYLE-GUIDE.md)
- [Tailwind CSS 文档](https://tailwindcss.com/docs)
- [Vue 3 文档](https://vuejs.org/)
- [TypeScript 文档](https://www.typescriptlang.org/)

---

**模板版本**：1.0.0
**最后更新**：2025-11-18
**适用项目**：Ripple IM App v0.1.0
