import { useEffect, useState } from 'react';
import styles from '../styles/home.module.css';
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
        <div className={styles.container}>
            <header className={styles.header}>
                <h1>📢 お知らせ一覧</h1>
                <p className={styles.subtitle}>バックエンド（Rust）経由でSupabaseから安全に取得しています</p>
            </header>

            {loading ? (
                <div className={styles.centerMessage}>
                    <div className={styles.spinner}></div>
                    <p>情報を読み込み中...</p>
                </div>
            ) : error ? (
                <p className={styles.centerMessage} style={{ color: '#ef4444' }}>{error}</p>
            ) : notices.length === 0 ? (
                <p className={styles.centerMessage}>現在、新しいお知らせはありません。</p>
            ) : (
                <div className={styles.noticeList}>
                    {notices.map((notice) => (
                        <article key={notice.id} className={styles.noticeCard}>
                            <div className={styles.cardMeta}>
                                <span className={styles.badge}>NEW</span>
                                <time className={styles.date}>
                                    {new Date(notice.created_at).toLocaleDateString('ja-JP', {
                                        year: 'numeric',
                                        month: '2-digit',
                                        day: '2-digit',
                                    })}
                                </time>
                            </div>
                            <h2 className={styles.cardTitle}>{notice.title}</h2>
                            <p className={styles.cardContent}>{notice.content}</p>
                        </article>
                    ))}
                </div>
            )}
        </div>
    );
}