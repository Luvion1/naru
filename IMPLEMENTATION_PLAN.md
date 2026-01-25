# Implementation Plan: Naru (Secure Configuration Manager)

## 📋 Executive Summary
- **Architecture:** Clean Architecture (DDD)
- **Stack:** .NET 10, C# 14, Postgres, Cloud Run
- **Testing Strategy:** TDD (xUnit + FluentAssertions + Moq)
- **Quality Standard:** **Production-Ready (Zero TODOs, Zero NotImplementedExceptions)**

## 🛠️ Phase 1: Project Scaffolding
*Goal: Initialize the repository, solution, and projects with strict hygiene.*

### Step 1.1: Initialize Solution
- **Command:** `dotnet new sln -n Naru`
- **Command:** `git init`
- **Action:** Create `.gitignore` using the standard `dotnet` template.

### Step 1.2: Create Projects
- **Command:** `dotnet new classlib -n Naru.Domain -o src/Naru.Domain`
- **Command:** `dotnet new classlib -n Naru.Application -o src/Naru.Application`
- **Command:** `dotnet new classlib -n Naru.Infrastructure -o src/Naru.Infrastructure`
- **Command:** `dotnet new webapi -n Naru.Api -o src/Naru.Api`
- **Command:** `dotnet new xunit -n Naru.UnitTests -o tests/Naru.UnitTests`
- **Command:** `dotnet new xunit -n Naru.IntegrationTests -o tests/Naru.IntegrationTests`

### Step 1.3: Project Hygiene (Cleanup)
- **Action:** Delete `Class1.cs` from Domain, Application, and Infrastructure projects.
- **Action:** Delete `UnitTest1.cs` from UnitTests and IntegrationTests projects.
- **Action:** Delete `WeatherForecast.cs` and `WeatherForecastController.cs` from Api project.
- **Verify:** Ensure `dotnet build` passes on an empty solution.

### Step 1.4: Configure Dependencies
- **Command:** `dotnet sln add (ls src/**/*.csproj) (ls tests/**/*.csproj)`
- **Command:** `dotnet add src/Naru.Application reference src/Naru.Domain`
- **Command:** `dotnet add src/Naru.Infrastructure reference src/Naru.Application`
- **Command:** `dotnet add src/Naru.Api reference src/Naru.Infrastructure`
- **Command:** `dotnet add tests/Naru.UnitTests reference src/Naru.Application`

---

## 🧪 Phase 2: Domain Layer (Core Logic)
*Goal: Implement the core configuration and schema logic using strict TDD.*

### Step 2.1: ConfigurationEntry Aggregate
- **Test:** Create `tests/Naru.UnitTests/Domain/ConfigurationEntryTests.cs`.
- **Action:** Define failing test for `ConfigurationEntry.Create(key, value, environment)`.
- **Implement:** Create `src/Naru.Domain/Aggregates/ConfigurationEntry.cs` (Sealed class, Primary Constructor).
- **Constraint:** Handle key validation (no special chars except `_` and `.`). **No placeholders.**

### Step 2.2: SchemaDefinition Value Object
- **Test:** Add tests for `SchemaDefinition.Validate(value)`.
- **Implement:** Create `src/Naru.Domain/ValueObjects/SchemaDefinition.cs`.
- **Constraint:** **MUST be fully implemented.** Support types: `String`, `Integer`, `Boolean`. Handle range validation for integers and length for strings.

### Step 2.3: Encryption Logic (Domain Service Interface)
- **Test:** Create `tests/Naru.UnitTests/Domain/SecretValueTests.cs`.
- **Action:** Define behavior for marking values as `IsSecret`.
- **Implement:** Create `src/Naru.Domain/ValueObjects/SecretValue.cs`.

---

## ⚙️ Phase 3: Infrastructure & Persistence
*Goal: Implement AES-GCM encryption and Postgres persistence.*

### Step 3.1: Encryption Service (AES-GCM)
- **Test:** Create `tests/Naru.UnitTests/Infrastructure/EncryptionServiceTests.cs`.
- **Implement:** `AesGcmEncryptionService` in `Naru.Infrastructure`.
- **Constraint:** Use `System.Security.Cryptography.AesGcm`. Ensure IV (Nonce) and Tag are handled correctly. **Zero TODOs.**

### Step 3.2: Postgres DbContext
- **Test:** Create Integration Test in `tests/Naru.IntegrationTests/Persistence/DbContextTests.cs` using `Testcontainers`.
- **Implement:** Create `NaruDbContext` inheriting from `DbContext`.
- **Action:** Configure `IEntityTypeConfiguration<ConfigurationEntry>` with proper Postgres column types (e.g., `jsonb` for metadata if needed).

---

## 🚀 Phase 4: Application Layer (MediatR)
*Goal: Orchestrate features via Handlers.*

### Step 4.1: SetConfiguration Command
- **Test:** Create `tests/Naru.UnitTests/Application/SetConfigurationHandlerTests.cs`.
- **Implement:** `SetConfigurationHandler` and `SetConfigurationCommand`.
- **Constraint:** Orchestrate: Load existing -> Validate Schema -> Encrypt if Secret -> Persist -> Log Audit.

### Step 4.2: Audit Logging Interceptor
- **Test:** Verify `AuditLog` entry is created whenever a configuration changes.
- **Implement:** `AuditLoggingBehavior<TRequest, TResponse>` (MediatR Pipeline Behavior).
- **Constraint:** Mask sensitive values in audit logs using `********`.

---

## 🌐 Phase 5: API & Deployment
*Goal: Expose endpoints and containerize.*

### Step 5.1: Configuration Endpoints
- **Test:** Create `tests/Naru.IntegrationTests/Api/ConfigurationApiTests.cs` (WebApplicationFactory).
- **Implement:** Minimal API endpoints in `Program.cs` for `POST /configs` and `GET /configs/{env}/{key}`.
- **Constraint:** Endpoints must return `201 Created` or `200 OK` with actual data. **No 501 Not Implemented.**

### Step 5.2: Dockerization (Cloud Run Ready)
- **Action:** Create `Dockerfile` in project root using multi-stage build.
- **Content:** Use `mcr.microsoft.com/dotnet/sdk:10.0-alpine` for build and `mcr.microsoft.com/dotnet/aspnet:10.0-alpine` for runtime.

---

## ✅ Success Criteria
1. All tests pass (`dotnet test` returns 0).
2. Solution builds with **Zero Warnings**.
3. **No default boilerplate files exist.**
4. **Codebase contains ZERO `NotImplementedException` or `//TODO` markers.**
5. Full audit trail exists for every "Set" operation.
