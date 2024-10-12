using SELLT = Skylark.Enum.LevelLogType;
using SELT = Skylark.Enum.LogType;
using SMHW = Sucrose.Manager.Helper.Writer;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMVL = Sucrose.Memory.Manage.Valuable.Log;
using SMR = Sucrose.Memory.Readonly;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;

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

            threadId = SMMRG.Randomise.Next(1000, 9999);

            logFilePath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Log, string.Format(logFileName, SMMVL.FileNameDate));

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

                SMHW.WriteBasic(logFilePath, $"[{SMMVL.FileTimeLine}] ~ [{SMMVL.FileDescriptionLine}-{threadId}/{level}] ~ [{message}]");
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
                    SMHW.WriteBasic(logFilePath, $"[{SMMVL.FileTimeLine}] ~ [{SMMVL.FileDescriptionLine}-{threadId}/{level}] ~ [{message}]");
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