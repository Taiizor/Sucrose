using SCLHP = Sucrose.CommandLine.Helper.Parse;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SWHWS = Skylark.Wing.Helper.WindowsStartup;

namespace Sucrose.CommandLine.Helper
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
                                        SSHC.Run(SCLHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Kill:
                                        SSHC.Kill(SCLHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Live:
                                        SSHC.Run(SCLHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Test:
                                        Console.WriteLine("Test Values:");

                                        foreach (string Value in Values)
                                        {
                                            Type Types = GetType(Value);

                                            if (Types == typeof(int))
                                            {
                                                Console.WriteLine(SCLHP.ArgumentValue<int>(Value));
                                            }
                                            else if (Types == typeof(bool))
                                            {
                                                Console.WriteLine(SCLHP.ArgumentValue<bool>(Value));
                                            }
                                            else if (Types == typeof(string))
                                            {
                                                Console.WriteLine(SCLHP.ArgumentValue<string>(Value));
                                            }
                                        }
                                        break;
                                    case SSECT.Report:
                                        SSHC.Run(SCLHP.ArgumentValue<string>(Values[0]));
                                        break;
                                    case SSECT.Startup:
                                        SWHWS.SetStartup(SCLHP.ArgumentValue<string>(Values[0]), SCLHP.ArgumentValue<string>(Values[1]), SCLHP.ArgumentValue<bool>(Values[2]));
                                        break;
                                    case SSECT.Interface:
                                        SSHC.Run(SCLHP.ArgumentValue<string>(Values[0]));
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

        private static Type GetType(string Variable)
        {
            if (bool.TryParse(Variable, out bool _))
            {
                return typeof(bool);
            }

            if (int.TryParse(Variable, out int _))
            {
                return typeof(int);
            }

            return typeof(string);
        }
    }
}