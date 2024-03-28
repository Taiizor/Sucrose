using Downloader;
using System.Net;
using SMMM = Sucrose.Manager.Manage.Manager;

namespace Sucrose.Wizard.Manage
{
    internal static class Internal
    {
        public static string Source = string.Empty;

        public static DownloadService DownloadService;

        public static readonly DownloadConfiguration DownloadConfiguration = new()
        {
            RangeLow = 0,
            RangeHigh = 0,
            ChunkCount = 4,
            Timeout = 1000,
            ParallelCount = 4,
            RangeDownload = false,
            BufferBlockSize = 4096,
            ParallelDownload = false,
            MaximumBytesPerSecond = 0,
            MaxTryAgainOnFailover = 5,
            MinimumSizeOfChunking = 1024,
            CheckDiskSizeBeforeDownload = true,
            ClearPackageOnCompletionWithFailure = true,
            MaximumMemoryBufferBytes = 1024 * 1024 * 50,
            ReserveStorageSpaceBeforeStartingDownload = true,
            RequestConfiguration = new()
            {
                Accept = "*/*",
                KeepAlive = true,
                UserAgent = SMMM.UserAgent,
                UseDefaultCredentials = false,
                ProtocolVersion = HttpVersion.Version11
            }
        };
    }
}