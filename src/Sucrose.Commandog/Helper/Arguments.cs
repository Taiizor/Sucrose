using SCHM = Sucrose.Commandog.Helper.Miscellaneous;
using SCHP = Sucrose.Commandog.Helper.Parse;
using SCHS = Sucrose.Commandog.Helper.Scheduler;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSHE = Sucrose.Space.Helper.Export;
using SSHI = Sucrose.Space.Helper.Import;
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

                            if (Enum.TryParse(Name, true, out SSECT Command))
                            {
                                switch (Command)
                                {
                                    case SSECT.Log:
                                        SSHC.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Kill:
                                        SSHC.Kill(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Live:
                                        SSHC.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Test:
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
                                    case SSECT.Temp:
                                        SSHT.Delete(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SSECT.Import:
                                        SSHI.Start(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]));
                                        break;
                                    case SSECT.Export:
                                        SSHE.Start(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Report:
                                        SSHC.Run(SCHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Startup:
                                        SWHWS.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SSECT.StartupM:
                                        SWHWSM.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<string>(Values[1]), SCHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SSECT.StartupP:
                                        SWHWSP.SetStartup(SCHP.ArgumentValue<string>(Values[0]), SCHP.ArgumentValue<bool>(Values[1]));
                                        break;
                                    case SSECT.Scheduler:
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
                                    case SSECT.Interface:
                                        SSHC.Run(SCHP.ArgumentValue<string>(Values[0]));
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