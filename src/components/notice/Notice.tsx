import { useState } from 'react';
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
    const [openNoticeId, setOpenNoticeId] = useState<string | null>(null);

    // クリック時に対象を開閉するハンドラー
    const toggleNotice = (id: string) => {
        setOpenNoticeId(openNoticeId === id ? null : id);
    };

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
                <div className="noticeAccordionList" style={{
                    display: 'flex',
                    flexDirection: 'column',
                    width: '100%'
                }}>
                    {notices.map((notice) => {
                        const isOpen = openNoticeId === notice.id;

                        return (
                            <article 
                                key={notice.id} 
                                className="noticeAccordionItem"
                                style={{
                                    borderBottom: '1px solid #e5e7eb',
                                    width: '100%',
                                }}
                            >
                                <div 
                                    onClick={() => toggleNotice(notice.id)}
                                    style={{
                                        display: 'flex',
                                        alignItems: 'center',
                                        padding: '1.25rem 0.5rem',
                                        cursor: 'pointer',
                                        gap: '1.5rem',
                                        userSelect: 'none',
                                        backgroundColor: isOpen ? '#f9fafb' : 'transparent', // 開いている時は少し背景色を変える
                                        transition: 'background-color 0.2s ease',
                                    }}
                                >
                                    {/* 日付とバッジのエリア */}
                                    <div style={{
                                        display: 'flex',
                                        alignItems: 'center',
                                        gap: '0.5rem',
                                        minWidth: '130px',
                                        flexShrink: 0
                                    }}>
                                        <span className="badge" style={{
                                            fontSize: '0.75rem',
                                            padding: '0.1rem 0.4rem'
                                        }}>
                                            NEW
                                        </span>
                                        <time className="date" style={{
                                            fontSize: '0.9rem',
                                            color: '#6b7280'
                                        }}>
                                            {new Date(notice.created_at).toLocaleDateString('ja-JP', {
                                                year: 'numeric',
                                                month: '2-digit',
                                                day: '2-digit',
                                            })}
                                        </time>
                                    </div>

                                    {/* タイトル表示エリア */}
                                    <div style={{ flexGrow: 1 }}>
                                        <h2 className="cardTitle" style={{
                                            fontSize: '1.05rem',
                                            margin: 0,
                                            fontWeight: isOpen ? 'bold' : 'normal',
                                            color: '#111827'
                                        }}>
                                            {notice.title}
                                        </h2>
                                    </div>

                                    <div style={{
                                        fontSize: '0.8rem',
                                        color: '#9ca3af',
                                        minWidth: '20px',
                                        textAlign: 'center'
                                    }}>
                                        {isOpen ? '▲' : '▼'}
                                    </div>
                                </div>

                                {isOpen && (
                                    <div 
                                        style={{
                                            padding: '0 0.5rem 1.5rem 11.5rem', // 日付エリアの横幅(130px + gap)に合わせて左余白を取り、縦ラインを綺麗に揃える
                                            backgroundColor: '#f9fafb',
                                        }}
                                    >
                                        <p 
                                            className="cardContent" 
                                            style={{ 
                                                fontSize: '0.95rem', 
                                                color: '#374151', 
                                                margin: 0, 
                                                lineHeight: '1.6',
                                                whiteSpace: 'pre-wrap' // 改行が崩れないように補正
                                            }}
                                            dangerouslySetInnerHTML={{ __html: notice.content }}
                                        >
                                        </p>
                                    </div>
                                )}
                            </article>
                        );
                    })}
                </div>
            )}
        </>
    );
}