import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

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

    return (
        <div className="container">
            <header className="header">
                <h1>📢 お知らせ一覧</h1>
                <p className="subtitle">バックエンド（Rust）経由でSupabaseから安全に取得しています</p>
            </header>

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