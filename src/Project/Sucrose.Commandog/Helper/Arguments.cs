using SCHM = Sucrose.Commandog.Helper.Miscellaneous;
using SCHP = Sucrose.Commandog.Helper.Parse;
using SCHS = Sucrose.Commandog.Helper.Scheduler;
using SDECT = Sucrose.Dependency.Enum.CommandsType;
using SMR = Sucrose.Memory.Readonly;
using SSHE = Sucrose.Space.Helper.Export;
using SSHI = Sucrose.Space.Helper.Import;
using SSHP = Sucrose.Space.Helper.Processor;
using SSHT = Sucrose.Space.Helper.Temporary;
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

                            if (Enum.TryParse(Name, true, out SDECT Command))
                            {
                                switch (Command)
                                {
                                    case SDECT.Log:
                                        SSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SDECT.Kill:
                                        SSHP.Kill(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SDECT.Live:
                                        SSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SDECT.Test:
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
                                    case SDECT.Temp:
                                        SSHT.Delete(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SDECT.Import:
                                        SSHI.Start(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SDECT.Export:
                                        SSHE.Start(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SDECT.Report:
                                        SSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SDECT.Startup:
                                        SWHWS.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SDECT.StartupM:
                                        SWHWSM.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SDECT.StartupP:
                                        SWHWSP.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<bool>(Values[1]));
                                        break;
                                    case SDECT.Scheduler:
                                        switch (SCHP.ArgumentValue<string>(Values[0]))
                                        {
                                            case "Create":
                                                SCHS.Create(SCHP.ArgumentValue<string>(Values[1]));
                                                break;
                                            case "Enable":
                                                SCHS.EnableTask();
                                                break;
                                            case "Disable":
                                                SCHS.DisableTask();
                                                break;
                                            case "Delete":
                                                SCHS.DeleteTask();
                                                break;
                                            default:
                                                break;
                                        }
                                        break;
                                    case SDECT.Interface:
                                        SSHP.Run(SCHP.ArgumentValue<string>(Values[0]));
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