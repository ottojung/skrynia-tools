{
  description = "Skrynia Tools: Rust CLI for the Skrynia HTTP server";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          package = pkgs.rustPlatform.buildRustPackage {
            pname = "skrynia-tools";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [
              "--bin"
              "skrynia-tools"
            ];
            # Socket-based integration tests run in the regular Rust CI.
            doCheck = false;
            meta = {
              description = "Skrynia HTTP client and application deployment CLI";
              homepage = "https://github.com/ottojung/skrynia-tools";
              license = pkgs.lib.licenses.agpl3Plus;
              mainProgram = "skrynia-tools";
            };
          };
        in
        {
          default = package;
          skrynia-tools = package;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          meta.description = "Skrynia Tools command-line interface";
          program = "${self.packages.${system}.default}/bin/skrynia-tools";
        };
      });
    };
}
