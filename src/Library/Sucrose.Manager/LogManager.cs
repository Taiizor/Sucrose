using SELLT = Skylark.Enum.LevelLogType;
using SELT = Skylark.Enum.LogType;
using SMHW = Sucrose.Manager.Helper.Writer;
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

            _lock = new ReaderWriterLockSlim(); //ReaderWriterLock
        }

        public void Log(SELLT level, string message)
        {
            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    if (logType == SELT.None)
                    {
                        return;
                    }

                    using Mutex Mutex = new(false, Path.GetFileName(logFilePath));

                    try
                    {
                        Mutex.WaitOne();

                        SMHW.WriteBasic(logFilePath, $"[{SMV.LogFileTime}] ~ [{SMR.LogDescription}-{threadId}/{level}] ~ [{message}]");
                    }
                    finally
                    {
                        Mutex.ReleaseMutex();
                    }
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }

        public bool CheckFile()
        {
            return File.Exists(logFilePath);
        }

        public string LogFile()
        {
            return logFilePath;
        }
    }
}