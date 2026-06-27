import { Notice } from "@/pages/index";

interface NoticeViewProps{
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
                            <p className="cardContent" dangerouslySetInnerHTML={{__html: notice.content}}></p>
                        </article>
                    ))}
                </div>
            )}
        </>
    );
}