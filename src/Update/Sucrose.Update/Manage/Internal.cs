using Downloader;
using System.Net;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;

namespace Sucrose.Update.Manage
{
    internal static class Internal
    {
        public static string Source = string.Empty;

        public static DownloadService DownloadService;

        public static readonly SSCEUT UpdateType = SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Compressed);

        public static readonly DownloadConfiguration DownloadConfiguration = new()
        {
            RangeLow = 0,
            RangeHigh = 0,
            ChunkCount = 8,
            Timeout = 1000,
            ParallelCount = 8,
            RangeDownload = false,
            BufferBlockSize = 8000,
            ParallelDownload = true,
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