/**
 * 파일 뷰어 타입 정의
 */
export type FileViewerType = 'text' | 'image' | 'media' | 'unsupported';

/**
 * 텍스트 파일 확장자 목록
 */
const TEXT_EXTENSIONS = [
  // 기본 텍스트
  '.txt', '.text', '.rtf',
  
  // 문서 형식
  '.md', '.markdown', '.mdown', '.mkd',
  '.rst', '.asciidoc', '.adoc',
  
  // 데이터 형식
  '.json', '.jsonl', '.ndjson',
  '.xml', '.xsl', '.xslt', '.svg',
  '.csv', '.tsv', '.dsv',
  '.yaml', '.yml', '.toml', '.ini', '.cfg', '.conf',
  
  // 로그 및 시스템 파일
  '.log', '.logs', '.out', '.err',
  '.pid', '.lock', '.tmp',
  
  // 웹 개발
  '.html', '.htm', '.xhtml',
  '.css', '.scss', '.sass', '.less',
  '.js', '.mjs', '.jsx', '.ts', '.tsx',
  '.vue', '.svelte',
  
  // 프로그래밍 언어
  '.c', '.h', '.cpp', '.cxx', '.cc', '.hpp',
  '.cs', '.vb', '.fs', '.fsx',
  '.java', '.kt', '.scala', '.groovy',
  '.py', '.pyw', '.pyi', '.ipynb',
  '.rb', '.rake', '.gemspec',
  '.php', '.phtml',
  '.go', '.mod', '.sum',
  '.rs', '.toml',
  '.swift', '.m', '.mm',
  '.dart', '.lua', '.pl', '.pm',
  '.r', '.R', '.Rmd',
  '.sql', '.mysql', '.pgsql', '.sqlite',
  
  // 쉘 스크립트
  '.sh', '.bash', '.zsh', '.fish', '.csh', '.tcsh',
  '.bat', '.cmd', '.ps1', '.psm1',
  
  // 설정 파일
  '.gitignore', '.gitattributes', '.gitmodules',
  '.dockerignore', '.editorconfig', '.eslintrc',
  '.prettierrc', '.babelrc', '.npmrc',
  '.env', '.env.local', '.env.production',
  
  // 빌드 도구
  '.makefile', '.cmake', '.gradle', '.sbt',
  '.package.json', '.composer.json', '.cargo.toml',
  '.requirements.txt', '.pipfile', '.poetry.lock',
  
  // 기타
  '.diff', '.patch', '.rej',
  '.license', '.readme', '.changelog', '.authors',
  '.todo', '.fixme', '.notes'
];

/**
 * 이미지 파일 확장자 목록
 */
const IMAGE_EXTENSIONS = [
  '.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.svg', '.ico', '.tiff', '.tif'
];

/**
 * 미디어 파일 확장자 목록
 */
const MEDIA_EXTENSIONS = [
  // 오디오 형식
  '.mp3', '.wav', '.ogg', '.aac', '.flac', '.m4a', '.wma', 
  '.aiff', '.aif', '.ape', '.opus', '.webm', '.3gp',
  
  // 비디오 형식  
  '.mp4', '.webm', '.avi', '.mov', '.mkv', '.flv', '.wmv', 
  '.m4v', '.3gp', '.ogv', '.ts', '.m3u8'
];

/**
 * 구문 강조 언어 매핑
 */
const SYNTAX_HIGHLIGHTING: Record<string, string[]> = {
  // 웹 기술
  'html': ['.html', '.htm', '.xhtml'],
  'css': ['.css', '.scss', '.sass', '.less'],
  'javascript': ['.js', '.mjs', '.jsx'],
  'typescript': ['.ts', '.tsx'],
  'vue': ['.vue'],
  'svelte': ['.svelte'],
  
  // 데이터 형식
  'json': ['.json', '.jsonl', '.ndjson'],
  'xml': ['.xml', '.xsl', '.xslt', '.svg'],
  'yaml': ['.yaml', '.yml'],
  'toml': ['.toml'],
  'ini': ['.ini', '.cfg', '.conf'],
  'csv': ['.csv', '.tsv'],
  
  // 마크업
  'markdown': ['.md', '.markdown', '.mdown', '.mkd'],
  'rst': ['.rst'],
  'asciidoc': ['.asciidoc', '.adoc'],
  
  // 프로그래밍 언어
  'c': ['.c', '.h'],
  'cpp': ['.cpp', '.cxx', '.cc', '.hpp'],
  'csharp': ['.cs'],
  'java': ['.java'],
  'python': ['.py', '.pyw', '.pyi'],
  'rust': ['.rs'],
  'go': ['.go'],
  'php': ['.php', '.phtml'],
  'ruby': ['.rb', '.rake', '.gemspec'],
  'swift': ['.swift'],
  'kotlin': ['.kt'],
  'scala': ['.scala'],
  'dart': ['.dart'],
  'lua': ['.lua'],
  'r': ['.r', '.R', '.Rmd'],
  'sql': ['.sql', '.mysql', '.pgsql', '.sqlite'],
  
  // 쉘 스크립트
  'bash': ['.sh', '.bash', '.zsh', '.fish'],
  'batch': ['.bat', '.cmd'],
  'powershell': ['.ps1', '.psm1'],
  
  // 설정 파일
  'dockerfile': ['Dockerfile', '.dockerfile'],
  'gitignore': ['.gitignore'],
  'makefile': ['Makefile', '.makefile'],
  'cmake': ['.cmake'],
  'gradle': ['.gradle'],
  
  // 기타
  'diff': ['.diff', '.patch'],
  'log': ['.log', '.logs', '.out', '.err']
};

/**
 * 파일 뷰어 타입을 감지합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 파일 뷰어 타입
 */
export function getFileViewerType(fileName: string | null | undefined, mimeType?: string): FileViewerType {
  if (!fileName || typeof fileName !== 'string' || fileName.trim() === '') {
    return 'unsupported';
  }
  
  try {
    const ext = getFileExtension(fileName);
    const lowerFileName = fileName.toLowerCase();
    
    // 확장자 없는 특수 파일들
    const specialTextFiles = [
      'readme', 'license', 'changelog', 'authors', 'contributors',
      'makefile', 'dockerfile', 'gemfile', 'rakefile', 'vagrantfile'
    ];
    
    if (specialTextFiles.some(name => lowerFileName.includes(name))) {
      return 'text';
    }
    
    // 확장자 기반 판단
    if (TEXT_EXTENSIONS.includes(ext)) {
      return 'text';
    }
    
    if (IMAGE_EXTENSIONS.includes(ext)) {
      return 'image';
    }
    
    if (MEDIA_EXTENSIONS.includes(ext)) {
      return 'media';
    }
    
    // MIME 타입 기반 판단
    if (mimeType) {
      if (mimeType.startsWith('text/')) return 'text';
      if (mimeType === 'application/json') return 'text';
      if (mimeType === 'application/xml') return 'text';
      if (mimeType === 'application/javascript') return 'text';
      if (mimeType.includes('script')) return 'text';
      
      if (mimeType.startsWith('image/')) return 'image';
      if (mimeType.startsWith('audio/') || mimeType.startsWith('video/')) return 'media';
    }
    
    return 'unsupported';
  } catch (error) {
    console.warn('파일 뷰어 타입 감지 중 오류:', error, '파일명:', fileName);
    return 'unsupported';
  }
}

/**
 * 구문 강조 언어를 감지합니다.
 * 
 * @param fileName - 파일명
 * @returns 구문 강조 언어
 */
export function getSyntaxLanguage(fileName: string | null | undefined): string {
  if (!fileName || typeof fileName !== 'string' || fileName.trim() === '') {
    return 'text';
  }
  
  try {
    const ext = getFileExtension(fileName);
    const lowerFileName = fileName.toLowerCase();
    
    // 특수 파일명 처리
    if (lowerFileName.includes('dockerfile')) return 'dockerfile';
    if (lowerFileName.includes('makefile')) return 'makefile';
    if (lowerFileName.includes('gemfile')) return 'ruby';
    if (lowerFileName.includes('rakefile')) return 'ruby';
    
    // 확장자 기반 언어 감지
    for (const [language, extensions] of Object.entries(SYNTAX_HIGHLIGHTING)) {
      if (extensions.includes(ext)) {
        return language;
      }
    }
    
    return 'text'; // 기본값
  } catch (error) {
    console.warn('구문 강조 언어 감지 중 오류:', error, '파일명:', fileName);
    return 'text';
  }
}

/**
 * 파일 확장자를 추출합니다.
 * 
 * @param fileName - 파일명
 * @returns 소문자 확장자 (점 포함)
 */
function getFileExtension(fileName: string | null | undefined): string {
  if (!fileName || typeof fileName !== 'string' || fileName.trim() === '') {
    return '';
  }
  
  try {
    const lastDot = fileName.lastIndexOf('.');
    return lastDot > 0 ? fileName.substring(lastDot).toLowerCase() : '';
  } catch (error) {
    console.warn('파일 확장자 추출 중 오류:', error, '파일명:', fileName);
    return '';
  }
}

/**
 * 파일이 텍스트 파일인지 확인합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 텍스트 파일 여부
 */
export function isTextFile(fileName: string | null | undefined, mimeType?: string): boolean {
  return getFileViewerType(fileName, mimeType) === 'text';
}

/**
 * 파일이 이미지 파일인지 확인합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 이미지 파일 여부
 */
export function isImageFile(fileName: string | null | undefined, mimeType?: string): boolean {
  return getFileViewerType(fileName, mimeType) === 'image';
}

/**
 * 파일이 미디어 파일인지 확인합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 미디어 파일 여부
 */
export function isMediaFile(fileName: string | null | undefined, mimeType?: string): boolean {
  return getFileViewerType(fileName, mimeType) === 'media';
}

/**
 * 지원되는 파일 형식인지 확인합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 지원 여부
 */
export function isSupportedFile(fileName: string | null | undefined, mimeType?: string): boolean {
  return getFileViewerType(fileName, mimeType) !== 'unsupported';
}

/**
 * 파일 크기를 사람이 읽기 쉬운 형태로 포맷합니다.
 * 
 * @param bytes - 바이트 크기
 * @returns 포맷된 크기 문자열
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * 파일 타입에 따른 아이콘을 반환합니다.
 * 
 * @param fileName - 파일명
 * @param mimeType - MIME 타입 (선택사항)
 * @returns 아이콘 문자열 (이모지)
 */
export function getFileIcon(fileName: string | null | undefined, mimeType?: string): string {
  if (!fileName || typeof fileName !== 'string' || fileName.trim() === '') {
    return '📁';
  }
  
  try {
    const viewerType = getFileViewerType(fileName, mimeType);
    const ext = getFileExtension(fileName);
    
    // 특정 확장자별 아이콘
    const iconMap: Record<string, string> = {
      // 문서
      '.pdf': '📄',
      '.doc': '📝', '.docx': '📝',
      '.xls': '📊', '.xlsx': '📊',
      '.ppt': '📽️', '.pptx': '📽️',
      
      // 이미지
      '.jpg': '🖼️', '.jpeg': '🖼️', '.png': '🖼️',
      '.gif': '🎞️', '.svg': '🎨',
      
      // 오디오
      '.mp3': '🎵', '.wav': '🎵', '.flac': '🎵',
      
      // 비디오
      '.mp4': '🎬', '.avi': '🎬', '.mkv': '🎬',
      
      // 압축
      '.zip': '📦', '.rar': '📦', '.7z': '📦',
      
      // 코드
      '.js': '📜', '.ts': '📜', '.py': '🐍',
      '.rs': '🦀', '.go': '🐹', '.java': '☕',
      
      // 설정
      '.json': '⚙️', '.xml': '⚙️', '.yaml': '⚙️', '.yml': '⚙️'
    };
    
    if (iconMap[ext]) {
      return iconMap[ext];
    }
    
    // 타입별 기본 아이콘
    switch (viewerType) {
      case 'text': return '📄';
      case 'image': return '🖼️';
      case 'media': return '🎵';
      default: return '📁';
    }
  } catch (error) {
    console.warn('파일 아이콘 감지 중 오류:', error, '파일명:', fileName);
    return '📁';
  }
}