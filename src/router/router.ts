import {createWebHistory, createRouter} from 'vue-router';
import Home from '../views/Home.vue';
import Login from '../views/Login.vue';

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/home',
            name: 'home',
            component: Home
        },
        {
            path: '/',
            name: 'login',
            component: Login
        },
    ]
});

export default router;