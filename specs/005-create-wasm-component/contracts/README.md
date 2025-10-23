# Contracts: WASM Component Creator Node

**Feature**: 005-create-wasm-component
**Date**: 2025-10-18

## Overview

This directory contains interface contracts for the WASM Component Creator feature. Since this is a desktop application without external APIs, contracts focus on internal module interfaces, WIT definitions, and component protocols.

## Contract Types

### 1. Internal Rust Module Interfaces
- **File**: `rust-module-interfaces.md`
- **Purpose**: Define function signatures and types for internal modules
- **Scope**: Creator node, compiler service, template generator, parser

### 2. WIT Component Contracts
- **File**: `generated-component-wit.wit`
- **Purpose**: Define WIT interface for generated user components
- **Scope**: Component metadata, execution, and UI interfaces

### 3. Structured Comment Protocol
- **File**: `comment-annotations.md`
- **Purpose**: Define annotation syntax and parsing rules
- **Scope**: User-facing DSL for port/capability specification

### 4. File System Contracts
- **File**: `filesystem-layout.md`
- **Purpose**: Define directory structure and file naming conventions
- **Scope**: Build workspaces, component storage, templates

## Contract Guarantees

### Stability Levels

- **STABLE**: Breaking changes require major version bump, migration guide
- **EVOLVING**: May change in minor versions with deprecation notice
- **EXPERIMENTAL**: No stability guarantees

### Current Status

| Contract | Stability | Version | Notes |
|----------|-----------|---------|-------|
| Rust Module Interfaces | EVOLVING | 0.1.0 | Internal APIs, can change |
| WIT Component Interface | STABLE | 1.0.0 | Must match existing components |
| Comment Annotations | STABLE | 1.0.0 | User-facing syntax |
| Filesystem Layout | EVOLVING | 0.1.0 | Implementation detail |

## Validation

### Contract Tests

All contracts MUST have corresponding tests:
- **Unit Tests**: Parse comment annotations, validate names
- **Integration Tests**: End-to-end compilation workflow
- **Contract Tests**: Generated components match WIT spec

### Test Coverage Requirements

- Comment parser: 100% (user-facing API)
- Template generator: >90%
- Compiler service: >80%
- Creator node UI: Manual/visual testing

## Related Documentation

- [data-model.md](../data-model.md): Entity definitions and state machines
- [research.md](../research.md): Technology decisions and rationale
- [spec.md](../spec.md): Feature requirements and user scenarios
