use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test Java support with a real Spring Boot structure
#[test]
#[ignore] // Requires jdtls installed
fn test_java_spring_boot_structure() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    // Create a minimal Spring Boot project structure
    fs::write(
        project.join("pom.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <groupId>com.example</groupId>
    <artifactId>demo</artifactId>
    <version>0.0.1-SNAPSHOT</version>
    <packaging>jar</packaging>
    
    <name>demo</name>
    <description>Demo project for TypeGlass</description>
    
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.2.0</version>
    </parent>
    
    <properties>
        <java.version>17</java.version>
    </properties>
    
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
        </dependency>
    </dependencies>
</project>
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/java/com/example/demo")).unwrap();
    
    // Create a controller with dependencies
    fs::write(
        project.join("src/main/java/com/example/demo/User.java"),
        r#"package com.example.demo;

public class User {
    private Long id;
    private String name;
    private String email;
    
    public User(Long id, String name, String email) {
        this.id = id;
        this.name = name;
        this.email = email;
    }
    
    public Long getId() { return id; }
    public String getName() { return name; }
    public String getEmail() { return email; }
}
"#,
    )
    .unwrap();
    
    fs::write(
        project.join("src/main/java/com/example/demo/UserService.java"),
        r#"package com.example.demo;

import java.util.List;
import java.util.ArrayList;

public class UserService {
    private List<User> users = new ArrayList<>();
    
    public User createUser(String name, String email) {
        User user = new User((long) users.size() + 1, name, email);
        users.add(user);
        return user;
    }
    
    public User findById(Long id) {
        return users.stream()
            .filter(u -> u.getId().equals(id))
            .findFirst()
            .orElse(null);
    }
    
    public List<User> findAll() {
        return new ArrayList<>(users);
    }
}
"#,
    )
    .unwrap();
    
    fs::write(
        project.join("src/main/java/com/example/demo/UserController.java"),
        r#"package com.example.demo;

import org.springframework.web.bind.annotation.*;
import java.util.List;

@RestController
@RequestMapping("/api/users")
public class UserController {
    private final UserService userService;
    
    public UserController(UserService userService) {
        this.userService = userService;
    }
    
    @GetMapping
    public List<User> getAllUsers() {
        return userService.findAll();
    }
    
    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }
    
    @PostMapping
    public User createUser(@RequestBody CreateUserRequest request) {
        return userService.createUser(request.name, request.email);
    }
    
    static class CreateUserRequest {
        public String name;
        public String email;
    }
}
"#,
    )
    .unwrap();
    
    // Test: Graph from UserController should include UserService and User
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("UserController")
        .arg("--depth")
        .arg("3");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("UserController"))
        .stdout(predicate::str::contains("UserService").or(predicate::str::contains("User")));
}

/// Test Kotlin support with Gradle project
#[test]
#[ignore] // Requires kotlin-language-server installed
fn test_kotlin_gradle_project() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    // Create Kotlin Gradle project
    fs::write(
        project.join("build.gradle.kts"),
        r#"plugins {
    kotlin("jvm") version "1.9.0"
}

group = "com.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib"))
}
"#,
    )
    .unwrap();
    
    fs::write(
        project.join("settings.gradle.kts"),
        r#"rootProject.name = "demo"
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/kotlin/com/example")).unwrap();
    
    fs::write(
        project.join("src/main/kotlin/com/example/User.kt"),
        r#"package com.example

data class User(
    val id: Long,
    val name: String,
    val email: String
)

class UserRepository {
    private val users = mutableListOf<User>()
    
    fun save(user: User): User {
        users.add(user)
        return user
    }
    
    fun findById(id: Long): User? =
        users.firstOrNull { it.id == id }
    
    fun findAll(): List<User> = users.toList()
}

class UserService(private val repository: UserRepository) {
    fun createUser(name: String, email: String): User {
        val user = User(
            id = repository.findAll().size.toLong() + 1,
            name = name,
            email = email
        )
        return repository.save(user)
    }
    
    fun getUser(id: Long): User? = repository.findById(id)
    
    fun getAllUsers(): List<User> = repository.findAll()
}
"#,
    )
    .unwrap();
    
    // Test: Graph from UserService should include UserRepository and User
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("UserService")
        .arg("--depth")
        .arg("2");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("UserService"));
}

/// Test real TypeScript project with complex dependencies
#[test]
#[ignore] // Requires typescript-language-server
fn test_typescript_complex_types() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("package.json"),
        r#"{
  "name": "complex-typescript",
  "version": "1.0.0",
  "dependencies": {
    "typescript": "^5.0.0"
  }
}
"#,
    )
    .unwrap();
    
    fs::write(
        project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true
  }
}
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src")).unwrap();
    
    // Complex TypeScript with generics, mapped types, conditional types
    fs::write(
        project.join("src/types.ts"),
        r#"// Generic Repository pattern
export interface Repository<T extends { id: string }> {
    findById(id: string): Promise<T | null>;
    findAll(): Promise<T[]>;
    save(entity: T): Promise<T>;
    delete(id: string): Promise<boolean>;
}

// User entity
export interface User {
    id: string;
    name: string;
    email: string;
    role: UserRole;
}

export type UserRole = 'admin' | 'user' | 'guest';

// Mapped type for partial updates
export type PartialUpdate<T> = {
    [P in keyof T]?: T[P];
};

// Conditional type for API responses
export type ApiResponse<T> = 
    T extends void 
    ? { success: true }
    : { success: true; data: T } | { success: false; error: string };

// User repository implementation
export class UserRepository implements Repository<User> {
    private users: User[] = [];
    
    async findById(id: string): Promise<User | null> {
        return this.users.find(u => u.id === id) ?? null;
    }
    
    async findAll(): Promise<User[]> {
        return [...this.users];
    }
    
    async save(user: User): Promise<User> {
        this.users.push(user);
        return user;
    }
    
    async delete(id: string): Promise<boolean> {
        const index = this.users.findIndex(u => u.id === id);
        if (index >= 0) {
            this.users.splice(index, 1);
            return true;
        }
        return false;
    }
    
    async update(id: string, updates: PartialUpdate<User>): Promise<User | null> {
        const user = await this.findById(id);
        if (!user) return null;
        
        Object.assign(user, updates);
        return user;
    }
}

// Service layer
export class UserService {
    constructor(private repository: UserRepository) {}
    
    async createUser(name: string, email: string, role: UserRole): Promise<ApiResponse<User>> {
        try {
            const user = await this.repository.save({
                id: Math.random().toString(36),
                name,
                email,
                role
            });
            return { success: true, data: user };
        } catch (error) {
            return { success: false, error: String(error) };
        }
    }
    
    async getUser(id: string): Promise<ApiResponse<User | null>> {
        const user = await this.repository.findById(id);
        return { success: true, data: user };
    }
}
"#,
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("UserService")
        .arg("--depth")
        .arg("3")
        .arg("--format")
        .arg("json");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("UserService"))
        .stdout(predicate::str::contains("Repository").or(predicate::str::contains("User")));
}
