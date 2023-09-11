using SCHM = Sucrose.Commandog.Helper.Miscellaneous;
using SCHP = Sucrose.Commandog.Helper.Parse;
using SCHS = Sucrose.Commandog.Helper.Scheduler;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDESCT = Sucrose.Shared.Dependency.Enum.SchedulerCommandsType;
using SSSHE = Sucrose.Shared.Space.Helper.Export;
using SSSHI = Sucrose.Shared.Space.Helper.Import;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHT = Sucrose.Shared.Space.Helper.Temporary;
using SWHWS = Skylark.Wing.Helper.WindowsStartup;
using SWHWSM = Skylark.Wing.Helper.WindowsStartupMachine;
using SWHWSP = Skylark.Wing.Helper.WindowsStartupPriority;

namespace Sucrose.Commandog.Helper
{
    internal static class Arguments
    {
        public static void Parse(string[] Arguments)
        {
            if (Arguments.Any())
            {
                for (int Count = 0; Count < Arguments.Length; Count++)
                {
                    string Argument = Arguments[Count];

                    if (Argument.StartsWith(SMR.StartCommand) && Argument.Contains(SMR.ValueSeparatorChar))
                    {

#if NET6_0_OR_GREATER
                        string[] ArgumentParts = Argument[1..].Split(SMR.ValueSeparatorChar);
#else
                        string[] ArgumentParts = Argument.Substring(1).Split(SMR.ValueSeparatorChar);
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
                                        SSSHT.Delete(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SSDECT.Wiki:
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSDECT.Bundle:
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSDECT.Update:
                                        SSSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSDECT.Import:
                                        SSSHI.Start(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SSDECT.Export:
                                        SSSHE.Start(SCHP.ArgumentValue<string>(Values[0]));
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
                                    case SSDECT.StartupM:
                                        SWHWSM.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SSDECT.StartupP:
                                        SWHWSP.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<bool>(Values[1]));
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
                                    default:
                                        break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}