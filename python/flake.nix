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
    coinvolution-py = (ps: with ps; (
      ps.buildPythonPackage rec {
        pname = "coinvolution";
        version = "0.0.1";
        src = ./.;
        pyproject = true;
        nativeBuildInputs = [ ps.hatchling ];
        pythonImportsCheck = [ pname ];
      }
    )) pkgs.python3Packages;
  in {
    packages.${system}.default = coinvolution-py;
    apps.${system}.solve-pzp = {
      type = "app";
      program = coinvolution-py.outputs.out.path + "/bin/solve-pzp";
    };
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        (pkgs.python3.withPackages (ps: (
          (python-dependencies ps) ++ [ coinvolution-py ]
        )))
      ];
    };
  };
}
