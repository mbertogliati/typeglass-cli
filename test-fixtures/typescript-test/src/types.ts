export interface User {
    id: string;
    name: string;
    email: string;
}

export class UserService {
    private users: Map<string, User> = new Map();
    
    getUser(id: string): User | undefined {
        return this.users.get(id);
    }
    
    createUser(user: User): void {
        this.users.set(user.id, user);
    }
}

export type UserRepository = {
    findById(id: string): Promise<User | null>;
    save(user: User): Promise<void>;
};
