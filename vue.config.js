const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin')

module.exports = {
  css: {
    loaderOptions: {
      postcss: {
        plugins: [
          require('postcss-nested')(),
          require('autoprefixer')({ grid: 'no-autoplace' }),
        ],
      },
    },
  },
  chainWebpack: config => {
    config
      .plugin('monaco-editor')
      .use(MonacoWebpackPlugin, [
        config.pluginOptions && config.pluginOptions.monaco,
      ])

    config.plugin('define').tap(definitions => {
      definitions[0]['process.env'].__VUE_PROD_DEVTOOLS__ = true
      return definitions
    })
  },
}
