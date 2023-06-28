using Sucrose.Memory;
using System;
using System.IO;
using System.Threading;

namespace Sucrose.Manager
{
    public class LogManager
    {
        private string logFilePath;
        private LogType logType = LogType.All;
        private static object lockObject = new();
        private readonly ReaderWriterLockSlim _lock;

        public LogManager(string logFileName, LogType logType = LogType.All)
        {
            logFilePath = Path.Combine(Readonly.AppDataPath, Readonly.AppName, Readonly.LogFolder, string.Format(logFileName, $"{DateTime.Now:yy.MM.dd}"));

            Directory.CreateDirectory(Path.GetDirectoryName(logFilePath));

            _lock = new ReaderWriterLockSlim();
        }

        public void Log(LogLevelType level, string message)
        {
            if (logType == LogType.None)
            {
                return;
            }

            _lock.EnterWriteLock();

            try
            {
                lock (lockObject)
                {
                    using StreamWriter writer = File.AppendText(logFilePath);
                    writer.WriteLine($"[{DateTime.Now:HH:mm:ss}] ~ [SucroseManager Thread/{level}] ~ [{message}]");
                }
            }
            finally
            {
                _lock.ExitWriteLock();
            }
        }
    }
}