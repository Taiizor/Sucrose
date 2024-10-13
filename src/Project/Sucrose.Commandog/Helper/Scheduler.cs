using Microsoft.Win32.TaskScheduler;
using System.Security.Principal;
using SMR = Sucrose.Memory.Readonly;
using Task = Microsoft.Win32.TaskScheduler.Task;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRS = Sucrose.Memory.Manage.Readonly.Scheduler;

namespace Sucrose.Commandog.Helper
{
    internal static class Scheduler
    {
        public static void Create(string Application)
        {
            using TaskService Service = new();

            TaskDefinition Definition = Service.NewTask();

            Definition.RegistrationInfo.Description = SMMRS.TaskDescription;

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

            Service.RootFolder.RegisterTaskDefinition(SMMRS.TaskName, Definition);
        }

        public static void CreateTask(string Application)
        {
            using TaskService Service = new();

            TaskFolder Folder = Service.RootFolder.CreateFolder(SMMRG.AppName, exceptionOnExists: false);

            TaskDefinition Definition = Service.NewTask();

            Definition.RegistrationInfo.Description = SMMRS.TaskDescription;

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

            Folder.RegisterTaskDefinition(SMMRS.TaskName, Definition);
        }

        public static bool Enable()
        {
            using TaskService Service = new();

            Task Task = Service.GetTask(SMMRS.TaskName);

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

            TaskFolder Folder = Service.GetFolder(SMMRG.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMMRS.TaskName, StringComparison.OrdinalIgnoreCase))
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

            Task Task = Service.GetTask(SMMRS.TaskName);

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

            TaskFolder Folder = Service.GetFolder(SMMRG.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMMRS.TaskName, StringComparison.OrdinalIgnoreCase))
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

            Task Task = Service.GetTask(SMMRS.TaskName);

            if (Task != null)
            {
                Service.RootFolder.DeleteTask(SMMRS.TaskName);

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

            TaskFolder Folder = Service.GetFolder(SMMRG.AppName);

            if (Folder != null)
            {
                bool Result = false;

                TaskCollection Collection = Folder.GetTasks();

                foreach (Task Task in Collection)
                {
                    if (Task.Name.Equals(SMMRS.TaskName, StringComparison.OrdinalIgnoreCase))
                    {
                        Folder.DeleteTask(SMMRS.TaskName);

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