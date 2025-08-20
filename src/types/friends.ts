export interface Friend {
  account: string;
  nickName: string;
  avatar?: string;
}

export interface FriendRequest {
  id: string;
  fromAccount: string;
  toAccount: string;
  fromNickName: string;
  fromAvatar?: string;
  status: 'pending' | 'accepted' | 'rejected';
  createdAt: string;
}

export type FriendRequestStatus = 'pending' | 'accepted' | 'rejected';