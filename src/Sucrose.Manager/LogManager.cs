using System.IO;
using System.Threading;
using SELLT = Skylark.Enum.LevelLogType;
using SELT = Skylark.Enum.LogType;
using SMR = Sucrose.Memory.Readonly;
using SMV = Sucrose.Memory.Valuable;

namespace Sucrose.Manager
{
    public class LogManager
    {
        private int threadId;
        private SELT logType;
        private string logFilePath;
        private static object lockObject = new();
        private readonly ReaderWriterLockSlim _lock;

        public LogManager(string logFileName, SELT logType = SELT.All)
        {
            this.logType = logType;

            threadId = SMR.Randomise.Next(1000, 9999);

            logFilePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.LogFolder, string.Format(logFileName, SMV.LogFileDate));

            Directory.CreateDirectory(Path.GetDirectoryName(logFilePath));

            _lock = new ReaderWriterLockSlim();
        }

        public void Log(SELLT level, string message)
        {
            if (logType == SELT.None)
            {
                return;
            }

            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    using StreamWriter writer = File.AppendText(logFilePath);
                    writer.WriteLine($"[{SMV.LogFileTime}] ~ [{SMR.LogDescription}-{threadId}/{level}] ~ [{message}]");
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }
    }
}