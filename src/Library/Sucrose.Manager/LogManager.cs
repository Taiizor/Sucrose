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

        public LogManager(string logFileName, SELT logType = SELT.All)
        {
            this.logType = logType;

            threadId = SMR.Randomise.Next(1000, 9999);

            logFilePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.LogFolder, string.Format(logFileName, SMV.LogFileDate));

            Directory.CreateDirectory(Path.GetDirectoryName(logFilePath));
        }

        public void Log(SELLT level, string message)
        {
            if (logType == SELT.None)
            {
                return;
            }

            using Mutex Mutex = new(false, Path.GetFileName(logFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                SMHW.WriteBasic(logFilePath, $"[{SMV.LogFileTime}] ~ [{SMR.LogDescription}-{threadId}/{level}] ~ [{message}]");
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public void Log(SELLT level, params string[] messages)
        {
            if (logType == SELT.None)
            {
                return;
            }

            using Mutex Mutex = new(false, Path.GetFileName(logFilePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch
                {
                    //
                }

                foreach (string message in messages)
                {
                    SMHW.WriteBasic(logFilePath, $"[{SMV.LogFileTime}] ~ [{SMR.LogDescription}-{threadId}/{level}] ~ [{message}]");
                }
            }
            finally
            {
                Mutex.ReleaseMutex();
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