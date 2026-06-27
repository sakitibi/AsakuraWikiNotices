import type { AppProps } from 'next/app';

export default function App({ Component, pageProps }: AppProps) {
    return (
        <>
            <link rel="stylesheet" href="https://sakitibi.github.io/static.asakurawiki.com/css/index.globals.css" />
            <style jsx global>{`
                body {
                    margin: 0;
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                    background-color: #f9fafb;
                    color: #111827;
                    -webkit-font-smoothing: antialiased;
                    -moz-osx-font-smoothing: grayscale;
                }
                /* ダークモード対応（OSの設定がダークモードの場合） */
                @media (prefers-color-scheme: dark) {
                    body {
                        background-color: #111827;
                        color: #f3f4f6;
                    }
                }
            `}</style>
            <Component {...pageProps} />
        </>
    );
}