# configure - pull in configuration from the environment

["The 12 Factor App"][12-factor] has this very good advice about managing
configuration:

> Store config in the environment

> An app’s config is everything that is likely to vary between deploys
> (staging, production, developer environments, etc). This includes:
> - Resource handles to the database, Memcached, and other backing services
> - Credentials to external services such as Amazon S3 or Twitter
> - Per-deploy values such as the canonical hostname for the deploy

For reasons laid out in the linked web page, it is poor practice to store this
information inline in the source code. Setting aside even their reasons, it is
especially inconvenient in compiled languages like Rust, as it requires
recompiling the entire project between deployment environments, and it means
performing an entire redeploy to change one of these values.

However, most libraries today take these kinds of configuration as an argument
to their constructor, leaving application authors responsible for developing
their own system for pulling those configurations from the environment.
Configure is an attempt to create a standardized way for libraries to pull
configuration from the environment, making it easier for end users to follow
best practices regarding configuration.

This doesn't apply to all kinds of "configuration." For example, if it is
performance critical that a configuration be applied at compile time (say, to
drive monomorphization in order to get inlining benefits), using this library
would not be appropriate. If the configuration could vary every time an
operation is performed (say because its configuring a kind of pretty printer or
formatter), using this library would probably not be appropriate either.

## The Configure trait

Libraries adopting this model should put the appropriate configuration all into
a struct:

```rust
struct Config {
    socket_addr: SocketAddr,
    tls_cert: Option<PathBuf>,
    // ... etc
}
```

This struct that needs to implement `Configure`. The easiest way to get
`Configure` implemented correctly is to derive it. Deriving `Configure`
requires the struct to also implement `Deserialize`, which can also be derived:

```rust
#[macro_use] extern crate configure;

extern crate serde;
#[macro_use] extern crate serde_derive;

#[derive(Deserialize, Configure)]
struct Config {
    socket_addr: SocketAddr,
    tls_cert: Option<PathBuf>,
    // ... etc
}
```

The Configure trait provides two functions:

- `Configure::generate`, a constructor which generates the configuration from
  the environment.
- `Configure::regenerate`, a method that updates the configuration by pulling
  from the environment again.

The generated implementation of `Configure` all pull the configuration from a
**configuration source**, which is controlled by the end application.

## Configuration sources

Ultimately, the end application controls the source from which configuration is
generation. This control is applied through the `CONFIGURATION` static. The
`Configure` impl will access the source using `CONFIGURATION.get`.

A default source is provided for you, so applications for which the default is
satisfactory don't need to do anything. Applications which want to store
configuration can override that source.

### The default source

By default, `configure` provides a source of configuration that users can rely
on. Users can use this source using the `use_default_config!` macro at the
beginning of their main function. The default source is targeted at network
services, and may not be appropriate to all other domains.

It works like this:

1. By default, it looks up configuration values using environment variables.
   For example, if the library `foo` had a config struct with the field `bar`,
   that field would be controlled by the `FOO_BAR` environment variable.
2. If no environment variable is set, it will fall back to looking in the
   `Cargo.toml`. If there is a `Cargo.toml` present, and it contains a
   `[package.metadata.foo]` table (where `foo` is the name of the library), the
   `bar` member of that table will control the `bar` field in `foo`'s config
   struct.

In general, it is recommended that most environments use env vars to control
configuration. The `Cargo.toml` fallback is intended for development
environments, so that you can check these values into the `Cargo.toml` and have
them be consistent across every developer's machine.

### Custom configuration source

Users can override the default configuration for their app using the
`use_config_from!` macro. This macro should only be invoked once, in the final
binary. First, you will need to prepare a type that implements `ConfigSource`
to be used as the source of configuration.

This allows users to control their configuration source without recompiling all
of the library code that depends on it, as would occur if the configuration
source were a type parameter.

[12-factor]: https://12factor.net/config
