import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import Head from 'next/head';

interface Notice {
    id: string;
    title: string;
    content: string;
    created_at: string;
}

interface Session {
    accessToken: string;
    refreshToken: string;
}

export default function Home() {
    const [notices, setNotices] = useState<Notice[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    // ログイン状態やトークンを表示・管理するためのステート
    const [loginInfo, setLoginInfo] = useState<Session | null>(null);

    const fetchNotices = async () => {
        try {
            setLoading(true);
            setError(null);
            
            const data = await invoke<Notice[]>('get_notices_from_supabase');
            setNotices(data);
        } catch (err) {
            console.error('データ取得失敗:', err);
            setError('お知らせの取得に失敗しました。');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        let unlisten: (() => void) | null = null;

        const setupDeepLink = async () => {
            try {
                unlisten = await listen<string>('deep-link-login', (event) => {
                    const urlStr = event.payload;
                    console.log('ディープリンクURLを受信しました:', urlStr);

                    try {
                        const searchParamsStr = urlStr.split('?')[1];
                        if (searchParamsStr) {
                            const params = new URLSearchParams(searchParamsStr);
                            const accessToken = params.get('access_token');
                            const refreshToken = params.get('refresh_token');

                            if (accessToken && refreshToken) {
                                setLoginInfo({ accessToken, refreshToken });
                                alert('ログインに成功しました！お知らせを取得します。');
                            }
                        }
                    } catch (parseErr) {
                        console.error('URLの解析に失敗しました:', parseErr);
                    }
                });
            } catch (err) {
                console.error('Tauri Event の初期化失敗:', err);
            }
        };

        setupDeepLink();

        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    useEffect(() => {
        if (loginInfo) {
            fetchNotices();
        }
    }, [loginInfo]);

    return (
        <>
            <Head>
                <title>お知らせ一覧</title>
                <link rel="stylesheet" href="https://sakitibi.github.io/static.asakurawiki.com/css/noticeapps/index.static.css" />
            </Head>
            <div className="container">
                <header className="header">
                    <h1>お知らせ一覧</h1>
                </header>

                {!loginInfo ? (
                    <div className="centerMessage" style={{ padding: '2rem', textAlign: 'center' }}>
                        <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>🔒</div>
                        <h2>ログインが必要です</h2>
                        <p style={{ color: '#6b7280', marginBottom: '1.5rem' }}>
                            このアプリを利用するには、ブラウザからログインしてください。
                        </p>
                    </div>
                ) : (
                    <>
                        <div style={{
                            background: '#10b981',
                            color: 'white',
                            padding: '1rem',
                            borderRadius: '8px',
                            marginBottom: '1.5rem',
                            wordBreak: 'break-all'
                        }}>
                            <h3>🔒 認証セッション有効</h3>
                            <p><strong>Access Token:</strong> {loginInfo.accessToken.substring(0, 15)}...</p>
                        </div>

                        {loading ? (
                            <div className="centerMessage">
                                <div className="spinner"></div>
                                <p>情報を読み込み中...</p>
                            </div>
                        ) : error ? (
                            <p className="centerMessage" style={{ color: '#ef4444' }}>{error}</p>
                        ) : notices.length === 0 ? (
                            <p className="centerMessage">現在、新しいお知らせはありません。</p>
                        ) : (
                            <div className="noticeList">
                                {notices.map((notice) => (
                                    <article key={notice.id} className="noticeCard">
                                        <div className="cardMeta">
                                            <span className="badge">NEW</span>
                                            <time className="date">
                                                {new Date(notice.created_at).toLocaleDateString('ja-JP', {
                                                    year: 'numeric',
                                                    month: '2-digit',
                                                    day: '2-digit',
                                                })}
                                            </time>
                                        </div>
                                        <h2 className="cardTitle">{notice.title}</h2>
                                        <p className="cardContent">{notice.content}</p>
                                    </article>
                                ))}
                            </div>
                        )}
                    </>
                )}
            </div>
        </>
    );
}