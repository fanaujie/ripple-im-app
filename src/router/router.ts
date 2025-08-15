import {createWebHistory, createRouter} from 'vue-router';
import Home from '../views/Home.vue';
import Login from '../views/Login.vue';
import ChatView from '../views/ChatView.vue';
import PeopleView from '../views/PeopleView.vue';
import SettingsView from '../views/SettingsView.vue';

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            name: 'login',
            component: Login
        },
        {
            path: '/home',
            name: 'home',
            component: Home,
            redirect: '/chat',
            children: [
                {
                    path: '/chat',
                    name: 'chat',
                    component: ChatView
                },
                {
                    path: '/people',
                    name: 'people',
                    component: PeopleView
                },
                {
                    path: '/settings',
                    name: 'settings',
                    component: SettingsView
                }
            ]
        },
    ]
});

export default router;