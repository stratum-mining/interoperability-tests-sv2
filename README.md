# sv2-compliance

A compliance testing tool for Stratum V2 implementations. It connects to your
SV2 endpoints and runs a growing suite of spec-conformance scenarios against
them, reporting pass/fail per test. Scenarios run against a single endpoint or,
via a TOML config file, across a batch of deployments.

## Usage

Test a single endpoint:

```sh
sv2-compliance --endpoint <host:port>
```

Test a batch of deployments described in a config file:

```sh
sv2-compliance --config targets.toml
```

Run `sv2-compliance --help` to see all available options. The tool exits with
code `0` when all tests pass and `1` otherwise.

## Config file

A config file describes the deployments you want to test. Each `[[site]]` entry
is one deployment. Only the addresses you provide are exercised:

- `pool_address` tests the endpoint as a pool.
- `jds_address` tests the endpoint as a Job Declarator Server.

Available fields per site:

- `name` (required): a display label shown in the results.
- `pool_address` (optional): the pool endpoint to test, as `host:port`.
- `jds_address` (optional): the Job Declarator Server endpoint to test, as
  `host:port`.
- `scenarios` (optional): a per-site allowlist. When present, only scenarios
  whose names partially match an entry (case-insensitively) run against this
  site. Omit it to run every registered scenario against the site.

A site with neither `pool_address` nor `jds_address` is skipped.


A more complete example is provided in `targets.example.toml`.
