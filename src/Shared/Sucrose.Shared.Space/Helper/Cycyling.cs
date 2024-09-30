using System.IO;
using SECNT = Skylark.Enum.ClearNumericType;
using SETT = Skylark.Enum.TimeType;
using SHN = Skylark.Helper.Numeric;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDETT = Sucrose.Shared.Dependency.Enum.TransitionType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSETTE = Skylark.Standard.Extension.Time.TimeExtension;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Cycyling
    {
        public static bool Check()
        {
            if (Directory.Exists(SMMM.LibraryLocation))
            {
                List<string> Themes = Directory.GetDirectories(SMMM.LibraryLocation).Select(Path.GetFileName).ToList();

                if (Themes.Any())
                {
                    Themes = Themes.Except(SMMM.DisableCycyling).ToList();

                    if (SMMM.Cycyling && (Themes.Count > 1 || (Themes.Count == 1 && !Themes.Contains(SMMM.LibrarySelected))) && SMMM.PassingCycyling >= Converter(SMMM.CycylingTime))
                    {
                        return true;
                    }
                    else
                    {
                        return false;
                    }
                }
                else
                {
                    return false;
                }
            }
            else
            {
                return false;
            }
        }

        public static void Change()
        {
            if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                if (Directory.Exists(SMMM.LibraryLocation))
                {
                    List<string> Themes = Directory.GetDirectories(SMMM.LibraryLocation).Select(Path.GetFileName).ToList();

                    if (Themes.Any())
                    {
                        string LibrarySelected = SMMM.LibrarySelected;

                        Themes = Themes.Where(Theme => !SMMM.DisableCycyling.Contains(Theme) || Theme == LibrarySelected).ToList();

                        if (Themes.Count > 1)
                        {
                            string Selected = string.Empty;

                            int Index = Themes.IndexOf(LibrarySelected);

                            switch (SSDMM.TransitionType)
                            {
                                case SSDETT.Random:
                                    while (string.IsNullOrEmpty(Selected))
                                    {
                                        while (Index == Themes.IndexOf(LibrarySelected))
                                        {
                                            Index = SMR.Randomise.Next(Themes.Count);
                                        }

                                        string Current = Themes[Index];

                                        string ThemePath = Path.Combine(SMMM.LibraryLocation, Current);
                                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                                        {
                                            SSTHI Info = SSTHI.ReadJson(InfoPath);

                                            if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                                            {
                                                Selected = Current;
                                            }
                                        }
                                    }
                                    break;
                                case SSDETT.Sequential:
                                    if (Index < 0 || Index >= Themes.Count)
                                    {
                                        Index = 0;
                                    }
                                    else
                                    {
                                        Index += 1;

                                        if (Index >= Themes.Count)
                                        {
                                            Index = 0;
                                        }
                                    }

                                    foreach (string Theme in Themes.Skip(Index))
                                    {
                                        string ThemePath = Path.Combine(SMMM.LibraryLocation, Theme);
                                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                                        {
                                            SSTHI Info = SSTHI.ReadJson(InfoPath);

                                            if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                                            {
                                                Selected = Theme;
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                default:
                                    break;
                            }

                            if (!string.IsNullOrEmpty(Selected))
                            {
                                SMMI.CyclingSettingManager.SetSetting(SMC.PassingCycyling, 0);

                                SMMI.LibrarySettingManager.SetSetting(SMC.LibrarySelected, Selected);

                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Cycyling}{SMR.ValueSeparator}{SMR.Unknown}");
                            }
                        }
                    }
                }
            }
        }

        private static int Converter(int Time)
        {
            return Convert.ToInt32(SHN.Numeral(SSETTE.Convert(Time, SETT.Minute, SETT.Second), false, false, 0, '0', SECNT.None));
        }
    }
}