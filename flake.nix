{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: 
  inputs.flake-utils.lib.eachDefaultSystem (system: let
    overlays = [ (import inputs.rust-overlay) ];

    pkgs = import inputs.nixpkgs {
      inherit system overlays;
      config.allowUnfree = true;
    };

    packages = with pkgs; [
      openssl
      pkg-config
      eza
      fd
      rust-bin.stable.latest.default
      cargo-watch
      sqlx-cli
    ];

    libraries = with pkgs; [
    ];
  in {
    devShell = with pkgs; mkShell {
      name = "prod-2025-promo-api";
      buildInputs = packages ++ libraries;
    
      DATABASE_URL = "postgres://postgres:postgres@localhost:5432/promo-code-backend-prod";
      DIRENV_LOG_FORMAT = "";
      LD_LIBRARY_PATH = "${lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH";
    };
  });
}

