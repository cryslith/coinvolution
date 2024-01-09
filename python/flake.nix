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
        propagatedBuildInputs = python-dependencies ps;
        pythonImportsCheck = [ pname ];
      }
    )) pkgs.python3Packages;
  in {
    packages.${system}.default = coinvolution-py;
    apps.${system}.solve-pzp = {
      type = "app";
      program = "${coinvolution-py}/bin/solve-pzp";
    };
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        # use this instead of loading coinvolution-py directly so z3 is in the path for the dev shell
        (pkgs.python3.withPackages (ps: [ coinvolution-py ]))
      ];
    };
  };
}
