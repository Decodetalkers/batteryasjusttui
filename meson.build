project('gdgtui', 'rust',
  version: '0.2.0',
  meson_version: '>= 1.0.0'
)

cargo = find_program('cargo', version: '>= 1.66')

rustc = find_program('rustc', version: '>= 1.66')

command = [ cargo , 'build']

targetdir = 'debug'

if not get_option('debug')
  command += '--release'
  targetdir = 'release'
endif

command += [
  '&&',
  'cp',
  meson.global_source_root() / 'target' / targetdir / meson.project_name(),
  '@OUTPUT@'
]

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')

gdgtui_target = custom_target(meson.project_name(),
  output: meson.project_name(),
  build_by_default: true,
  install: true,
  install_dir: bindir,
  console: true,
  command: command
)
