use crate::ripple_api::api_response::RelationUser;

pub mod relation_operation {
    pub const ADD_FRIEND: u64 = 1;
    pub const UPDATE_FRIEND_REMARK_NAME: u64 = 2;
    pub const DELETE_FRIEND: u64 = 3;
    pub const ADD_BLOCK: u64 = 4;
    pub const DELETE_BLOCK: u64 = 5;
    pub const UNBLOCK_RESTORE_FRIEND: u64 = 6;
    pub const HIDE_BLOCK: u64 = 7;
    pub const UPDATE_FRIEND_NICK_NAME: u64 = 8;
    pub const UPDATE_FRIEND_AVATAR: u64 = 9;
    pub const BLOCK_STRANGER: u64 = 10;
    pub const UPDATE_FRIEND_INFO: u64 = 11;
}

#[derive(Debug)]
pub enum RelationAction {
    Upsert(RelationUser),
    UpdateRemarkName {
        user_id: String,
        remark_name: String,
    },
    UpdateNickName {
        user_id: String,
        nick_name: String,
    },
    UpdateAvatar {
        user_id: String,
        avatar: Option<String>,
    },
    UpdateFlags {
        user_id: String,
        flags: i32,
    },
    Delete {
        user_id: String,
    },
    UpdateNickNameAvatar {
        user_id: String,
        nick_name: String,
        avatar: Option<String>,
    },
}

pub mod relation_event_action {
    pub const ADD_FRIEND: i32 = 0;
    pub const REMOVE_FRIEND: i32 = 1;
    pub const UPDATE_FRIEND: i32 = 2;
    pub const ADD_BLOCK: i32 = 3;
    pub const REMOVE_BLOCK: i32 = 4;
    pub const BLOCK_FRIEND: i32 = 5;
    pub const UNBLOCK_TO_FRIEND: i32 = 6;
    pub const CLEAR: i32 = -1;
}
