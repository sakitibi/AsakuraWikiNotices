import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

interface Notice {
    id: string;
    title: string;
    content: string;
    created_at: string;
}

export default function Home() {
    const [notices, setNotices] = useState<Notice[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // ログイン状態やトークンを表示・管理するためのステートを追加
    const [loginInfo, setLoginInfo] = useState<{ accessToken: string; refreshToken: string } | null>(null);

    // 1. 既存のお知らせ取得ロジック
    useEffect(() => {
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

        fetchNotices();
    }, []);

    // 2. ディープリンク（カスタムプロトコル）を待ち受けるロジックを追加
    useEffect(() => {
        let unlisten: (() => void) | null = null;

        const setupDeepLink = async () => {
            try {
                unlisten = await listen<string>('deep-link-login', (event) => {
                    const urlStr = event.payload;
                    console.log('ディープリンクURLを受信しました:', urlStr);

                    try {
                        // URLからクエリパラメータ文字列を切り出す
                        const searchParamsStr = urlStr.split('?')[1];
                        if (searchParamsStr) {
                            const params = new URLSearchParams(searchParamsStr);
                            const accessToken = params.get('access_token');
                            const refreshToken = params.get('refresh_token');

                            if (accessToken && refreshToken) {
                                // 🔑 トークンをステートに格納
                                setLoginInfo({ accessToken, refreshToken });
                                
                                // ここに各種ログイン完了処理を記述します
                                // 例: Supabaseの認証セッションに反映する場合
                                // supabase.auth.setSession({ access_token: accessToken, refresh_token: refreshToken });
                                
                                alert('ディープリンク経由でログイン情報を取得しました！');
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

        // クリーンアップ（画面が閉じたらリスナーを解除）
        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    return (
        <div className="container">
            <header className="header">
                <h1>📢 お知らせ一覧</h1>
            </header>

            {loginInfo && (
                <div style={{
                    background: '#10b981',
                    color: 'white',
                    padding: '1rem',
                    borderRadius: '8px',
                    marginBottom: '1.5rem',
                    wordBreak: 'break-all'
                }}>
                    <h3>🔒 ログインに成功しました！</h3>
                    <p><strong>Access Token:</strong> {loginInfo.accessToken.substring(0, 15)}...</p>
                    <p><strong>Refresh Token:</strong> {loginInfo.refreshToken.substring(0, 15)}...</p>
                </div>
            )}

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
        </div>
    );
}