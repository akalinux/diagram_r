import path from 'path';
import webpack from 'webpack'
import MiniCssExtractPlugin from 'mini-css-extract-plugin';
//import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";

export default {
    devServer: {
        static: [
            {
                directory: path.join(__dirname, "public"),
                publicPath: "/",
            },
            {
                directory: path.join(__dirname, "public", "images"),
                publicPath: "/images",
            },
            {
                directory: path.join(__dirname, "public", "js"),
                publicPath: "/images",
            },
        ],
        compress: true,
        port: 5000,
    },
    mode: 'development',
    watchOptions: {
        poll: 1000,
    },
    entry: './src/index.ts',
    plugins: [
        new MiniCssExtractPlugin({
            filename: '[name].css',
            chunkFilename: '[id].css',
        }),
        /*new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "..", ".."),
            extraArgs: "--target web",
        }),*/
    ],

    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'asset/resource',
                generator: {
                    filename: '[name][ext]',
                }
            },
            {
                test: /\.css$/i,
                use: [
                    // Use MiniCssExtractPlugin.loader for production, style-loader for development
                    //process.env.NODE_ENV === 'production' ? MiniCssExtractPlugin.loader : 'style-loader',
                    { loader: 'style-loader' },
                    {
                        loader: 'css-loader',
                        options: {
                            // Set modules to false for third-party CSS to prevent CSS Modules behavior
                            modules: false,
                        },
                    },
                ],
            },
            {
                test: /\.tsx?$/,
                exclude: /node_modules/,
                use: 'ts-loader',
            }
        ],
    },

    resolve: {
        extensions: ['.ts', '.tsx', '.js', '.jsx'],
        alias: {
            react: path.resolve('./node_modules/react'),
            'react-dom': path.resolve('./node_modules/react-dom'),
        },
    },
    devtool: 'source-map',
    output: {
        path: path.resolve(__dirname, '.', 'public', 'js'),
        filename: 'main.js',
        clean: true,
    },
    experiments: {
        asyncWebAssembly: true
    }
};
