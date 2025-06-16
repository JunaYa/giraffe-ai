# Product Requirements: GitHub-like Permission System Web Application

## 1. Overview

A modern web application for managing fine-grained permissions in collaborative software development, inspired by GitHub's model. The system supports organizations, teams, repositories, users, roles, and advanced branch protection, with both RBAC and ABAC models.

## 2. Core Features

### 2.1 User Management

- **User Registration & Authentication**: Users can register, log in, and manage their profiles.
- **User Roles**: Assign roles at organization, team, and repository levels.
- **User Invitation**: Invite users to organizations, teams, or repositories.
- **User Activity Tracking**: View user actions and permission changes.

### 2.2 Organization Management

- **Organization Creation**: Users can create organizations.
- **Organization Roles**: Owner, Member, Moderator, Billing Admin, Security Admin, App Admin.
- **Custom Organization Roles**: Enterprise users can define custom roles with specific permissions.
- **Organization Settings**: Manage org-wide settings, billing, security, and integrations.

### 2.3 Team Management

- **Team Creation & Nesting**: Create teams, support nested (hierarchical) teams.
- **Team Membership**: Add/remove users, assign team roles.
- **Team Permissions**: Grant teams access to repositories with specific roles.
- **Permission Inheritance**: Sub-teams inherit permissions from parent teams.

### 2.4 Repository Management

- **Repository Creation**: Create repositories under personal or organization accounts.
- **Repository Roles**: Admin, Maintainer, Writer, Triager, Reader.
- **Direct & Indirect Access**: Assign users/teams directly or via org/team roles.
- **Repository Visibility**: Public/private settings.
- **Repository Settings**: Manage webhooks, deploy keys, topics, etc.

### 2.5 Branch Protection & ABAC

- **Branch Protection Rules**: Define rules for branches (e.g., require PR review, status checks, signed commits, linear history).
- **Pattern Matching**: Apply rules to branches matching patterns.
- **Attribute-Based Access Control (ABAC)**: Enforce rules based on request/commit/PR attributes.

### 2.6 Permission Evaluation

- **RBAC Engine**: Role-based permission checks for all entities.
- **ABAC Engine**: Attribute-based checks for dynamic, context-aware policies.
- **Permission Aggregation**: Aggregate permissions from direct, team, and org roles; always grant the highest effective permission.
- **Permission Inheritance**: Support recursive team/org permission inheritance.
- **Audit Trail**: Log all permission changes and access checks.

### 2.7 API

- **RESTful & OpenAPI**: Expose all core features via RESTful APIs, documented with OpenAPI (Utoipa).
- **Authentication**: JWT-based authentication for all endpoints.
- **Authorization**: All endpoints protected by permission checks.
- **Reflection/Introspection**: API to query available roles, permissions, and effective access for a user.

### 2.8 UI/UX

- **Three-Column Layout**:
  - **Left Sidebar**: Server, service, and method selection (excluding reflection service).
  - **Middle Sidebar**: Dynamic input form generated from request schema.
  - **Main Content**: Request/response history, collapsible panels, streaming support.
- **Header Input**: Add/remove custom headers via dynamic rows.
- **Request Form**: Auto-generate form fields from input schema; support multiple requests for streaming.
- **Response Display**: Show response as JSON, support streaming responses (list of values).
- **History Management**: Collapse all but latest request panel on new request.

### 2.9 Security & Observability

- **Audit Logging**: All permission changes and sensitive actions are logged.
- **Structured Logging**: Use structured, queryable logs for all API actions.
- **Health Checks**: Expose health endpoints for monitoring.
- **Metrics**: Collect and expose metrics for API usage, errors, and performance.

## 3. Usage Scenarios

- **Invite a user to a team, which inherits permissions from a parent team, granting access to multiple repositories.**
- **Define a branch protection rule requiring two PR reviews and passing CI before merge.**
- **Query a user's effective permissions on a repository, considering all direct and indirect roles.**
- **Enterprise admin creates a custom role with specific billing and security permissions.**

## 4. Non-Functional Requirements

- **Performance**: Low-latency permission checks (<10ms typical).
- **Scalability**: Support thousands of orgs, teams, and repos.
- **Security**: All endpoints require authentication and authorization.
- **Extensibility**: Support for future custom roles and ABAC policies.
- **Reliability**: High availability, robust error handling, and auditability.

## 5. Glossary

- **RBAC**: Role-Based Access Control
- **ABAC**: Attribute-Based Access Control
- **Org**: Organization
- **Repo**: Repository
- **PR**: Pull Request
