import { Notice } from "@/pages/index";

interface NoticeViewProps {
    fetchError: string | null;
    notices: Notice[];
    loadingNotices: boolean;
}

export default function NoticeView({
    fetchError,
    notices,
    loadingNotices
}: NoticeViewProps) {
    return (
        <>
            {loadingNotices ? (
                <div className="centerMessage">
                    <div className="spinner"></div>
                    <p>情報を読み込み中...</p>
                </div>
            ) : fetchError ? (
                <p className="centerMessage" style={{ color: '#ef4444' }}>{fetchError}</p>
            ) : notices.length === 0 ? (
                <p className="centerMessage">現在、新しいお知らせはありません。</p>
            ) : (
                <div className="noticeListStyle" style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>
                    {notices.map((notice) => (
                        <article 
                            key={notice.id} 
                            className="noticeListItem"
                            style={{
                                display: 'flex',
                                alignItems: 'baseline', // 日付、タイトル、中身の縦位置を揃える
                                padding: '1rem 0.5rem',
                                borderBottom: '1px solid #e5e7eb', // 行ごとの区切り線
                                gap: '1.5rem', // 各要素の間隔
                                width: '100%'
                            }}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', minWidth: '130px', flexShrink: 0 }}>
                                <span className="badge" style={{ fontSize: '0.75rem', padding: '0.1rem 0.4rem' }}>NEW</span>
                                <time className="date" style={{ fontSize: '0.9rem', color: '#6b7280' }}>
                                    {new Date(notice.created_at).toLocaleDateString('ja-JP', {
                                        year: 'numeric',
                                        month: '2-digit',
                                        day: '2-digit',
                                    })}
                                </time>
                            </div>

                            <div style={{ flexGrow: 1 }}>
                                <h2 className="cardTitle" style={{ fontSize: '1.05rem', margin: '0 0 0.25rem 0', fontWeight: 'bold' }}>
                                    {notice.title}
                                </h2>
                                <p 
                                    className="cardContent" 
                                    style={{ fontSize: '0.9rem', color: '#4b5563', margin: 0, lineHeight: '1.5' }}
                                    dangerouslySetInnerHTML={{ __html: notice.content }}
                                >
                                </p>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </>
    );
}