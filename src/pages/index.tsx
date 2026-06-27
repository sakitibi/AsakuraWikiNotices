import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import Head from 'next/head';
import NoticeView from '@/components/notice/Notice';

export interface Notice {
    id: string;
    title: string;
    content: string;
    created_at: string;
}

interface SupabaseUser {
    id: string;
    email?: string;
    user_metadata?: any;
}

function useSession() {
    const [user, setUser] = useState<SupabaseUser | null>(null);
    const [status, setStatus] = useState<'loading' | 'authenticated' | 'unauthenticated'>('loading');
    const [error, setError] = useState<string | null>(null);

    const checkInitialSession = async () => {
        try {
            const userData = await invoke<SupabaseUser>('verify_supabase_session');
            setUser(userData);
            setStatus('authenticated');
            setError(null);
        } catch (err) {
            console.log("[Session] 保存されたセッションがないか、無効です。");
            setUser(null);
            setStatus('unauthenticated');
        }
    };

    const loginWithCode = async (code: string) => {
        setStatus('loading');
        setError(null);
        try {
            const userData = await invoke<SupabaseUser>('exchange_code_for_session', { code });
            setUser(userData);
            setStatus('authenticated');
        } catch (err: any) {
            console.error(err);
            setStatus('unauthenticated');
            setError(err.toString() || '認証コードが正しくないか、有効期限が切れています。');
        }
    };

    // 初回起動時にファイルからロード
    useEffect(() => {
        checkInitialSession();
    }, []);

    const logout = async () => {
        await invoke('clear_supabase_session').catch(() => {});
        setUser(null);
        setStatus('unauthenticated');
        setError(null);
    };

    return { user, status, error, logout, loginWithCode };
}

export default function Home() {
    const { user, status, error: sessionError, logout, loginWithCode } = useSession();
    const [notices, setNotices] = useState<Notice[]>([]);
    const [loadingNotices, setLoadingNotices] = useState(false);
    const [fetchError, setFetchError] = useState<string | null>(null);
    
    const [pinCode, setPinCode] = useState('');

    const loadNotices = async () => {
        try {
            setLoadingNotices(true);
            setFetchError(null);
            const data = await invoke<Notice[]>('get_notices_from_supabase');
            setNotices(data);
        } catch (err) {
            setFetchError('お知らせの取得に失敗しました。');
        } finally {
            setLoadingNotices(false);
        }
    };

    useEffect(() => {
        if (status === 'authenticated') {
            loadNotices();
        } else {
            setNotices([]);
        }
    }, [status]);

    const handleCodeSubmit = (e: React.SubmitEvent) => {
        e.preventDefault();
        if (pinCode.trim().length === 8) {
            loginWithCode(pinCode.trim());
        }
    };

    return (
        <>
            <Head>
                <title>お知らせ一覧</title>
                <link rel="stylesheet" href="https://sakitibi.github.io/static.asakurawiki.com/css/noticeapps/index.static.css" />
            </Head>
            <div className="container">
                <header className="header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <h1>お知らせ一覧</h1>
                    <br/>
                    {status === 'authenticated' && (
                        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                            <span style={{ fontSize: '0.9rem', color: '#4b5563' }}>👤 {user?.email}</span>
                            <button onClick={logout} style={{ padding: '0.5rem 1rem', background: '#ef4444', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>
                                <span>ログアウト</span>
                            </button>
                        </div>
                    )}
                </header>

                {status === 'loading' && (
                    <div className="centerMessage">
                        <div className="spinner"></div>
                        <p>セッション情報を検証中...</p>
                    </div>
                )}

                {status === 'unauthenticated' && (
                    <div className="centerMessage" style={{ padding: '2rem', textAlign: 'center' }}>
                        <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>🔒</div>
                        <h2>ログインが必要です</h2>
                        <p style={{ color: '#6b7280', marginBottom: '1.5rem' }}>
                            ブラウザで発行された8桁の認証コードを30秒以内に入力してください。
                        </p>

                        <form onSubmit={handleCodeSubmit} style={{ display: 'flex', justifyContent: 'center', gap: '0.5rem', marginBottom: '1.5rem' }}>
                            <input
                                type="text"
                                placeholder="Ab123Xyz"
                                maxLength={8}
                                value={pinCode}
                                onChange={(e) => setPinCode(e.target.value.replace(/[^0-9a-zA-Z]/g, ''))}
                                style={{ 
                                    padding: '0.5rem', 
                                    fontSize: '1.2rem', 
                                    letterSpacing: '0.2rem', 
                                    textAlign: 'center', 
                                    width: '160px',
                                    border: '1px solid #d1d5db', 
                                    borderRadius: '4px',
                                }}
                            />
                            <button 
                                type="submit" 
                                disabled={pinCode.length !== 8} 
                                style={{
                                    padding: '0.5rem 1.2rem',
                                    background: pinCode.length === 8 ? '#2563eb' : '#9ca3af',
                                    color: 'white',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: pinCode.length === 8 ? 'pointer' : 'default',
                                    fontWeight: 'bold'
                                }}
                            >
                                <span>認証</span>
                            </button>
                        </form>

                        {sessionError && <p style={{ color: '#ef4444', marginTop: '1rem' }}>{sessionError}</p>}
                    </div>
                )}

                {status === 'authenticated' && <NoticeView
                    fetchError={fetchError}
                    notices={notices}
                    loadingNotices={loadingNotices}
                />}
            </div>
        </>
    );
}