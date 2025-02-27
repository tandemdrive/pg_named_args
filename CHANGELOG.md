# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.3.0] - 2025-02-27

Add support for SQL fragments that can be combined dynamically.

Re-export postgres-types from pg_named_args so that it does not need 
to be added to the dependencies.

Allow some lints for users of `query_args`.

## [0.2.3] - 2024-03-26

Fix: Improve error messages when sql contains invalid parameter groups.

## [0.2.2] - 2024-02-28

Fix: The macro would incorrectly accept struct update syntax.
Copied the readme documentation to the docs.

## [0.2.1] - 2024-02-06

Fix: The macro would incorrectly accept some invocations without `Args` struct name.

## [0.2.0] - 2024-01-30

The actual first working version of `pg_named_arg`!

## [0.1.0] - 2024-01-30

Initial release without any functionality yet, just to claim the name at
crates.io.
