{
  description = "test shell";

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    python-dependencies = ps: with ps; [
      jsonschema
      quart
      z3
    ];
    coinvolution-py = ps: with ps; (
      ps.buildPythonPackage {
        pname = "coinvolution";
        version = "0.0.1";
        src = ./src;
        pyproject = true;
        nativeBuildInputs = [ ps.hatchling ];
      }
    );
  in {
    packages.${system}.default = (coinvolution-py pkgs.python3Packages);
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        (pkgs.python3.withPackages (ps: (
          (python-dependencies ps) ++ [ (coinvolution-py ps) ]
        )))
      ];
    };
  };
}
