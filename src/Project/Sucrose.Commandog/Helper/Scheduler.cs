using Microsoft.Win32.TaskScheduler;
using System.Security.Principal;
using SMR = Sucrose.Memory.Readonly;
using Task = Microsoft.Win32.TaskScheduler.Task;

namespace Sucrose.Commandog.Helper
{
    internal static class Scheduler
    {
        public static void Create(string Application)
        {
            using TaskService Service = new();

            TaskDefinition Definition = Service.NewTask();

            Definition.RegistrationInfo.Description = SMR.TaskDescription;

            Definition.Actions.Add(new ExecAction(Application));

            LogonTrigger Trigger = new()
            {
                UserId = WindowsIdentity.GetCurrent().Name
            };

            Definition.Triggers.Add(Trigger);

            TaskSettings Settings = Definition.Settings;

            Settings.StopIfGoingOnBatteries = false;
            Settings.DisallowStartIfOnBatteries = false;

            Settings.ExecutionTimeLimit = TimeSpan.Zero;

            Service.RootFolder.RegisterTaskDefinition(SMR.TaskName, Definition);
        }

        public static void CreateTask(string Application)
        {
            using TaskService Service = new();

            TaskFolder Folder = Service.RootFolder.CreateFolder(SMR.AppName, exceptionOnExists: false);

            TaskDefinition Definition = Service.NewTask();

            Definition.RegistrationInfo.Description = SMR.TaskDescription;

            Definition.Actions.Add(new ExecAction(Application));

            LogonTrigger Trigger = new()
            {
                UserId = WindowsIdentity.GetCurrent().Name
            };

            Definition.Triggers.Add(Trigger);

            TaskSettings Settings = Definition.Settings;

            Settings.StopIfGoingOnBatteries = false;
            Settings.DisallowStartIfOnBatteries = false;

            Settings.ExecutionTimeLimit = TimeSpan.Zero;

            Folder.RegisterTaskDefinition(SMR.TaskName, Definition);
        }

        public static bool Enable()
        {
            using TaskService Service = new();

            Task Task = Service.GetTask(SMR.TaskName);

            if (Task != null)
            {
                Task.Enabled = true;
                Task.Definition.Settings.Enabled = true;
                Task.RegisterChanges();

                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool EnableTask()
        {
            using TaskService Service = new();

            TaskFolder Folder = Service.GetFolder(SMR.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMR.TaskName, StringComparison.OrdinalIgnoreCase))
                    {
                        Task.Enabled = true;
                        Task.Definition.Settings.Enabled = true;
                        Task.RegisterChanges();

                        Result = true;
                    }
                }

                return Result;
            }
            else
            {
                return false;
            }
        }

        public static bool Disable()
        {
            using TaskService Service = new();

            Task Task = Service.GetTask(SMR.TaskName);

            if (Task != null)
            {
                Task.Enabled = false;
                Task.Definition.Settings.Enabled = false;
                Task.RegisterChanges();

                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool DisableTask()
        {
            using TaskService Service = new();

            TaskFolder Folder = Service.GetFolder(SMR.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMR.TaskName, StringComparison.OrdinalIgnoreCase))
                    {
                        Task.Enabled = false;
                        Task.Definition.Settings.Enabled = false;
                        Task.RegisterChanges();

                        Result = true;
                    }
                }

                return Result;
            }
            else
            {
                return false;
            }
        }

        public static bool Delete()
        {
            using TaskService Service = new();

            Task Task = Service.GetTask(SMR.TaskName);

            if (Task != null)
            {
                Service.RootFolder.DeleteTask(SMR.TaskName);

                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool DeleteTask()
        {
            using TaskService Service = new();

            TaskFolder Folder = Service.GetFolder(SMR.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMR.TaskName, StringComparison.OrdinalIgnoreCase))
                    {
                        Folder.DeleteTask(SMR.TaskName);

                        Result = true;
                    }
                }

                return Result;
            }
            else
            {
                return false;
            }
        }
    }
}