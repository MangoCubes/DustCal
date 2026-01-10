{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Things needed at build time only, something that doesn't have to exist on the system when running the compiled program
        # `pkg-config` is a helper tool used when compiling applications and libraries, so it is only needed when the program is being compiled
        nativeBuildInputs = [ pkgs.pkg-config ];
        # Things needed at runtime, something that must be installed on the computer in order to run the program
        # `OpenSSL` library is a library that handles a bunch of cryptographic operations, and is expected to be available on the device when the program is running
        # `pkg-config` fetches `OpenSSL` installed on the system, and uses it during compilation
        buildInputs = [
          pkgs.openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          packages = (
            with pkgs;
            [
              rustup

              # This package is needed to ensure subsequent shells looks pretty
              bashInteractive
            ]
          );
          shellHook =
            let
              initFile = pkgs.writeText ".bashrc" ''
                echo "Activating Rust develop environment..."
                set -a
                  hw() { echo "Hello world!"; }
                set +a
              '';
            in
            ''
              bash --init-file ${initFile}; exit
            '';
        };
      }
    );
}
