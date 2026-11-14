-- Toi Yêu Yuri Online Judge (tyyoj) Initial Schema
-- PostgreSQL 17

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(32) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'user',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verification_token VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS problems (
    id UUID PRIMARY KEY,
    code VARCHAR(32) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    difficulty VARCHAR(16) NOT NULL DEFAULT 'Medium',
    statement_markdown TEXT NOT NULL,
    time_limit_ms BIGINT NOT NULL DEFAULT 1000,
    memory_limit_mb BIGINT NOT NULL DEFAULT 256,
    judge_mode VARCHAR(32) NOT NULL DEFAULT 'Standard',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS test_cases (
    id UUID PRIMARY KEY,
    problem_id UUID NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    input_data TEXT NOT NULL,
    expected_output TEXT NOT NULL,
    is_sample BOOLEAN NOT NULL DEFAULT FALSE,
    order_num INT NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS contests (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    rule_type VARCHAR(16) NOT NULL DEFAULT 'ICPC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    member_count INT NOT NULL DEFAULT 0,
    contests_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS submissions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    problem_id UUID NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    language VARCHAR(16) NOT NULL,
    source_code TEXT NOT NULL,
    verdict VARCHAR(32) NOT NULL DEFAULT 'Pending',
    time_consumed_ms BIGINT NOT NULL DEFAULT 0,
    memory_consumed_kb BIGINT NOT NULL DEFAULT 0,
    compile_log TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS submission_tests (
    id UUID PRIMARY KEY,
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    test_case_id UUID REFERENCES test_cases(id) ON DELETE CASCADE,
    test_order INT NOT NULL,
    verdict VARCHAR(32) NOT NULL,
    time_ms BIGINT NOT NULL DEFAULT 0,
    memory_kb BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS blogs (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    author_name VARCHAR(64) NOT NULL,
    content_markdown TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS wikis (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    category VARCHAR(64) NOT NULL,
    content_markdown TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial real problems and test cases into database
INSERT INTO problems (id, code, title, difficulty, statement_markdown, time_limit_ms, memory_limit_mb, judge_mode)
VALUES
    ('a0000000-0000-0000-0000-000000000001', 'P1001', 'A + B Problem', 'Easy',
     '### Bài toán A + B
Cho hai số nguyên $a, b \in \mathbb{Z}$ thỏa mãn $-10^9 \le a, b \le 10^9$. Hãy tính giá trị của tổng $S = a + b$.

**Định dạng đầu vào:** Một dòng chứa hai số nguyên $a$ và $b$ cách nhau bởi dấu cách.
**Định dạng đầu ra:** In ra một số nguyên duy nhất là kết quả $a + b$.',
     1000, 256, 'Standard'),

    ('a0000000-0000-0000-0000-000000000002', 'P1002', 'Số Fibonacci Lớn', 'Medium',
     '### Số Fibonacci Lớn
Cho số nguyên dương $n$ ($1 \le n \le 10^{18}$). Hãy tính:
$$ F_n \pmod{10^9 + 7} $$
với $F_1 = 1, F_2 = 1, F_n = F_{n-1} + F_{n-2}$.',
     1500, 512, 'Standard'),

    ('a0000000-0000-0000-0000-000000000003', 'P1003', 'Đoán số bí mật (Interactive)', 'Hard',
     '### Đoán số bí mật (Interactive)
Giám khảo chọn một số nguyên bí mật $X \in [1, 10^9]$. Bạn được phép hỏi tối đa 32 câu hỏi dạng `? Y`.
Giám khảo sẽ trả lời `<`, `>` hoặc `=`.
In ra `! X` khi đã tìm được số $X$. Luôn xả bộ đệm (flush buffer) sau mỗi câu hỏi.',
     2000, 256, 'Interactive')
ON CONFLICT (code) DO NOTHING;

-- Seed real test cases for P1001
INSERT INTO test_cases (id, problem_id, input_data, expected_output, is_sample, order_num)
VALUES
    ('b0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', E'3 5\n', E'8\n', TRUE, 1),
    ('b0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001', E'-10 20\n', E'10\n', FALSE, 2),
    ('b0000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000001', E'1000000000 1000000000\n', E'2000000000\n', FALSE, 3)
ON CONFLICT (id) DO NOTHING;

-- Seed real Contests
INSERT INTO contests (id, title, description, start_time, end_time, rule_type)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'Toi Yêu Yuri Contest #01 (ICPC)', 'Kỳ thi chính thức mở màn hệ thống tyyoj theo luật ICPC penalty time.', NOW() - INTERVAL '1 hour', NOW() + INTERVAL '4 hours', 'ICPC'),
    ('c0000000-0000-0000-0000-000000000002', 'Yuri Grand Contest #02 (IOI)', 'Kỳ thi thuật toán tính điểm theo từng subtask.', NOW() + INTERVAL '2 days', NOW() + INTERVAL '2 days 5 hours', 'IOI')
ON CONFLICT (id) DO NOTHING;

-- Seed real Organizations
INSERT INTO organizations (id, name, description, member_count, contests_count)
VALUES
    ('d0000000-0000-0000-0000-000000000001', 'Cộng đồng VNOI', 'Cộng đồng Lập trình Thi đấu Việt Nam', 15200, 240),
    ('d0000000-0000-0000-0000-000000000002', 'Đại học Công nghệ - ĐHQGHN (VNU-UET)', 'Khoa Công nghệ Thông tin, CLB Thuật toán UET', 480, 18),
    ('d0000000-0000-0000-0000-000000000003', 'Yuri CP Enthusiasts', 'Câu lạc bộ Lập trình thi đấu Toi Yêu Yuri', 1250, 12)
ON CONFLICT (id) DO NOTHING;

-- Seed real Blogs
INSERT INTO blogs (id, title, author_name, content_markdown)
VALUES
    ('e0000000-0000-0000-0000-000000000001', 'Chào mừng đến với Toi Yêu Yuri Online Judge', 'toiyeuyuriiiii',
     'Toi Yêu Yuri Online Judge chính thức vận hành trên nền tảng Podman containerized judgers và Nginx reverse proxy.'),
    ('e0000000-0000-0000-0000-000000000002', 'Hướng dẫn giải bài tập tương tác (Interactive)', 'nmkdeveloper',
     'Bài viết tổng hợp các lỗi thường gặp khi làm bài tập tương tác, nguyên nhân gây ra `IDLENESS_LIMIT_EXCEEDED` và cách xả bộ đệm `fflush(stdout)`.')
ON CONFLICT (id) DO NOTHING;

-- Seed real Wiki
INSERT INTO wikis (id, title, category, content_markdown)
VALUES
    ('f0000000-0000-0000-0000-000000000001', 'Cờ biên dịch và Môi trường', 'Compiler',
     'C++20 sử dụng `g++ -std=c++20 -O2`. Pascal sử dụng `fpc -O2`. Python sử dụng `python3`.'),
    ('f0000000-0000-0000-0000-000000000002', 'Các chế độ chấm trong tyyoj', 'Judger',
     'Hệ thống hỗ trợ 3 chế độ: Standard Diff, Custom Checker (testlib.h) và Interactive (POSIX pipes).')
ON CONFLICT (id) DO NOTHING;
