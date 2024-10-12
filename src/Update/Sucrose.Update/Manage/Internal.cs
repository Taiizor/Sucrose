using Downloader;
using System.IO;
using System.Net;
using SMMG = Sucrose.Manager.Manage.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using Timer = System.Timers.Timer;

namespace Sucrose.Update.Manage
{
    internal static class Internal
    {
        public static bool Trying = false;

        public static string Source = string.Empty;

        public static DownloadService DownloadService;

        public static bool Chance = SMMRG.Randomise.Next(2) == 0;

        public static string CachePath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMR.CacheFolder, SMR.Bundle);

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
                UserAgent = SMMG.UserAgent,
                UseDefaultCredentials = false,
                ProtocolVersion = HttpVersion.Version11
            }
        };
    }
}