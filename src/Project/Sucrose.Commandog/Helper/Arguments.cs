using SCHM = Sucrose.Commandog.Helper.Miscellaneous;
using SCHP = Sucrose.Commandog.Helper.Parse;
using SCHS = Sucrose.Commandog.Helper.Scheduler;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDESCT = Sucrose.Shared.Dependency.Enum.SchedulerCommandType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHD = Sucrose.Shared.Space.Helper.Delete;
using SSSHE = Sucrose.Shared.Space.Helper.Export;
using SSSHI = Sucrose.Shared.Space.Helper.Import;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHR = Sucrose.Shared.Space.Helper.Reset;
using SSSHT = Sucrose.Shared.Space.Helper.Temporary;
using SWHWS = Skylark.Wing.Helper.WindowsStartup;
using SWHWSM = Skylark.Wing.Helper.WindowsStartupMachine;
using SWHWSP = Skylark.Wing.Helper.WindowsStartupPriority;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;

namespace Sucrose.Commandog.Helper
{
    internal static class Arguments
    {
        public static async Task Parse(string[] Arguments)
        {
            if (Arguments.Any())
            {
                string Combined = string.Join(" ", Arguments);

                if (Combined.StartsWith(SMR.StartCommand) && Combined.Contains(SMR.ValueSeparatorChar))
                {

#if NET6_0_OR_GREATER
                    string[] ArgumentParts = Combined[1..].Split(SMR.ValueSeparatorChar);
#else
                    string[] ArgumentParts = Combined.Substring(1).Split(SMR.ValueSeparatorChar);
#endif

                    if (ArgumentParts.Length >= 2)
                    {
                        string Name = ArgumentParts[0];

#if NET6_0_OR_GREATER
                        List<string> Values = new(ArgumentParts[1..]);
#else
                        List<string> Values = new();

                        for (int Index = 1; Index < ArgumentParts.Length; Index++)
                        {
                            Values.Add(ArgumentParts[Index]);
                        }
#endif

                        if (Enum.TryParse(Name, true, out SSDECT Command))
                        {
                            switch (Command)
                            {
                                case SSDECT.Log:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Kill:
                                    SSSHP.Kill(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Live:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Test:
                                    Console.WriteLine("Test Values:");

                                    foreach (string Value in Values)
                                    {
                                        Type Types = SCHM.GetType(Value);

                                        if (Types == typeof(int))
                                        {
                                            Console.WriteLine(SCHP.ArgumentValue<int>(Value));
                                        }
                                        else if (Types == typeof(bool))
                                        {
                                            Console.WriteLine(SCHP.ArgumentValue<bool>(Value));
                                        }
                                        else if (Types == typeof(string))
                                        {
                                            Console.WriteLine(SCHP.ArgumentValue<string>(Value));
                                        }
                                    }
                                    break;
                                case SSDECT.Temp:
                                    await SSSHT.Delete(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    break;
                                case SSDECT.Wiki:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Reset:
                                    await SSSHR.Start(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Bundle:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Update:
                                    if (Values.Count() > 1)
                                    {
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    }
                                    else
                                    {
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    }
                                    break;
                                case SSDECT.Import:
                                    await SSSHI.Start(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    break;
                                case SSDECT.Export:
                                    await SSSHE.Start(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    break;
                                case SSDECT.Report:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Publish:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Startup:
                                    SWHWS.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                    break;
                                case SSDECT.Cycyling:
                                    SSLHK.Stop();

                                    SSLHK.StopSubprocess();

                                    SSLHR.Start();

                                    await Task.Delay(1500);

                                    if (!SSSHL.Run())
                                    {
                                        SSLHR.Start();
                                    }
                                    break;
                                case SSDECT.Property:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.StartupM:
                                    SWHWSM.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                    break;
                                case SSDECT.StartupP:
                                    SWHWSP.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<bool>(Values[1]));
                                    break;
                                case SSDECT.Official:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Watchdog:
                                    if (Values.Count() == 2)
                                    {
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    }
                                    break;
                                case SSDECT.Interface:
                                    if (Values.Count() > 1)
                                    {
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    }
                                    else
                                    {
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    }
                                    break;
                                case SSDECT.PropertyA:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                    break;
                                case SSDECT.Reportdog:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), string.Empty);
                                    break;
                                case SSDECT.Scheduler:
                                    switch (SCHP.ArgumentValue<SSDESCT>(Values[0]))
                                    {
                                        case SSDESCT.Create:
                                            SCHS.CreateTask(SCHP.ArgumentValue<string>(Values[1]));
                                            break;
                                        case SSDESCT.Enable:
                                            SCHS.EnableTask();
                                            break;
                                        case SSDESCT.Disable:
                                            SCHS.DisableTask();
                                            break;
                                        case SSDESCT.Delete:
                                            SCHS.DeleteTask();
                                            break;
                                        default:
                                            break;
                                    }
                                    break;
                                case SSDECT.Repository:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.Versioning:
                                    SSSHD.Folder(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                    break;
                                case SSDECT.Discussions:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                    break;
                                case SSDECT.RestartLive:
                                    SSLHK.Stop();

                                    SSLHK.StopSubprocess();

                                    SSSHP.Kill(SMMRA.Backgroundog);

                                    SSLHR.Start();

                                    await Task.Delay(1500);

                                    if (!SSSHL.Run())
                                    {
                                        SSLHR.Start();
                                    }
                                    break;
                                case SSDECT.Backgroundog:
                                    SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]), string.Empty);
                                    break;
                                default:
                                    break;
                            }
                        }
                    }
                }
            }

            await Task.CompletedTask;
        }
    }
}