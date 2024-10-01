using Downloader;
using System.Net;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using Timer = System.Timers.Timer;

namespace Sucrose.Update.Manage
{
    internal static class Internal
    {
        public static bool Trying = false;

        public static string Source = string.Empty;

        public static DownloadService DownloadService;

        public static bool Chance = SMR.Randomise.Next(2) == 0;

        public static readonly SSCEUT UpdateType = SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Executable);

        public static Timer Checker = new()
        {
            Enabled = false,
            Interval = 1000,
            AutoReset = true
        };

        public static Timer Limiter = new()
        {
            Enabled = false,
            Interval = 1000,
            AutoReset = true
        };

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
                KeepAlive = false,
                UserAgent = SMMM.UserAgent,
                UseDefaultCredentials = false,
                ProtocolVersion = HttpVersion.Version11
            }
        };
    }
}