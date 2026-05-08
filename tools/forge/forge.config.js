require('dotenv').config();

const path = require('path');
const packageJson = require('./../../package.electron.json');
const rootDir = process.cwd();

const linuxIconPng = path.join(rootDir, 'assets/build/icon_512.png');

module.exports = {
  packagerConfig: {
    asar: true,
    executableName: process.platform === 'linux' ? packageJson.name : packageJson.productName,
    icon: path.join(rootDir, 'assets/build/icon'),
    appBundleId: packageJson.appId,
    ...(process.argv.includes('--no-sign')
      ? {}
      : {
        osxSign: {},
        osxNotarize: {
          tool: 'notarytool',
          appleId: process.env.APPLE_ID,
          appleIdPassword: process.env.APPLE_PASSWORD,
          teamId: process.env.APPLE_TEAM_ID,
        },
      }),
  },
  makers: [
    {
      name: '@electron-forge/maker-dmg',
      config: {
        overwrite: true,
        background: path.join(rootDir, 'assets/build/background-dmg.png'),
        icon: path.join(rootDir, 'assets/build/icon.icns'),
        additionalDMGOptions: {
          window: { size: { width: 660, height: 500 } }
        },
      }
    },
    {
      name: "@electron-forge/maker-squirrel",
      config: {
        setupIcon: path.join(rootDir, 'assets/build/icon.ico'),
        iconUrl: 'https://raw.githubusercontent.com/heza-ru/Lumos/main/assets/build/icon.ico',
        loadingGif: path.join(rootDir, 'assets/build/loading.gif'),
        name: 'Lumos',
        shortcutName: 'Lumos - Electron',
        setupExe: 'Lumos.Setup.exe',
        noMsi: true
      }
    },
    {
      name: "@electron-forge/maker-deb",
      config: {
        options: {
          icon: linuxIconPng,
          categories: ['Graphics', 'Utility'],
          maintainer: "heza-ru",
          homepage: 'https://github.com/heza-ru/Lumos'
        }
      }
    },
    {
      name: "@electron-forge/maker-zip",
      platforms: ["darwin", "linux", "win32"]
    }
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-webpack',
      config: {
        devContentSecurityPolicy: `default-src 'self' 'unsafe-inline' data:; script-src 'self' 'unsafe-inline' data:`,
        mainConfig: path.join(rootDir, 'tools/webpack/main.js'),
        renderer: {
          config: path.join(rootDir, 'tools/webpack/renderer.js'),
          entryPoints: [
            {
              name: 'app_window',
              html: path.join(rootDir, 'src-electron/renderer/app_page/index.html'),
              js: path.join(rootDir, 'src-electron/renderer/app_page/index.js'),
              preload: {
                js: path.join(rootDir, 'src-electron/renderer/app_page/preload.js'),
              },
            },
            {
              name: 'extended_toolbar_window',
              html: path.join(rootDir, 'src-electron/renderer/extended_toolbar_page/index.html'),
              js: path.join(rootDir, 'src-electron/renderer/extended_toolbar_page/index.js'),
              preload: {
                js: path.join(rootDir, 'src-electron/renderer/extended_toolbar_page/preload.js'),
              },
            },
            {
              name: 'about_window',
              html: path.join(rootDir, 'src-electron/renderer/about_page/index.html'),
              js: path.join(rootDir, 'src-electron/renderer/about_page/index.js'),
              preload: {
                js: path.join(rootDir, 'src-electron/renderer/about_page/preload.js'),
              },
            },
            {
              name: 'settings_window',
              html: path.join(rootDir, 'src-electron/renderer/settings_page/index.html'),
              js: path.join(rootDir, 'src-electron/renderer/settings_page/index.js'),
              preload: {
                js: path.join(rootDir, 'src-electron/renderer/settings_page/preload.js'),
              },
            },
          ]
        },
        devServer: {
          liveReload: false,
        },
      }
    }
  ],
  publishers: [
    {
      name: '@electron-forge/publisher-github',
      config: {
        repository: {
          owner: 'heza-ru',
          name: 'Lumos',
        },
        draft: true
      }
    }
  ]
};
