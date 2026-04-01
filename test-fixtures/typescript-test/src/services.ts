import { User, UserService, UserRepository } from './types';

export class UserManager {
    constructor(
        private service: UserService,
        private repository: UserRepository
    ) {}
    
    async loadUser(id: string): Promise<User | null> {
        const cached = this.service.getUser(id);
        if (cached) return cached;
        
        return await this.repository.findById(id);
    }
}
