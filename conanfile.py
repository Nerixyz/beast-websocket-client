from conan import ConanFile
from conan.tools.files import copy
from os import path
import shutil


class Eventsub(ConanFile):
    name = "Eventsub"
    requires = [
        "boost/[~1.83]",
        "openssl/[~3]",
    ]
    settings = "os", "compiler", "build_type", "arch"
    default_options = {
        "openssl*:shared": True,
        "boost*:header_only": True,
    }

    options = {}
    generators = "CMakeDeps", "CMakeToolchain"

    def layout(self):
        self.cpp.build.libdirs = ["lib"]
        self.cpp.build.bindirs = ["bin"]

    def requirements(self):
        self.output.warning(self.default_options)

    def generate(self):
        for dep in self.dependencies.values():
            try:
                # macOS
                copy(
                    self,
                    "*.dylib",
                    dep.cpp_info.libdirs[0],
                    path.join(self.build_folder, self.cpp.build.libdirs[0]),
                    keep_path=False,
                )
                # Windows
                copy(
                    self,
                    "*.lib",
                    dep.cpp_info.libdirs[0],
                    path.join(self.build_folder, self.cpp.build.libdirs[0]),
                    keep_path=False,
                )
                copy(
                    self,
                    "*.dll",
                    dep.cpp_info.bindirs[0],
                    path.join(self.build_folder, self.cpp.build.bindirs[0]),
                    keep_path=False,
                )
                # Linux
                copy(
                    self,
                    "*.so*",
                    dep.cpp_info.libdirs[0],
                    path.join(self.build_folder, self.cpp.build.libdirs[0]),
                    keep_path=False,
                )
            except shutil.SameFileError:
                # Ignore 'same file' errors
                pass
