using Newtonsoft.Json.Linq;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SECNT = Skylark.Enum.ClearNumericType;
using SESP = Skylark.Enum.SimilarPasswordType;
using SHN = Skylark.Helper.Numeric;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEPPE = Skylark.Standard.Extension.Password.PasswordExtension;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Graphic
    {
        public static bool Condition()
        {
            return Condition(SBMI.GraphicData.Amd) || Condition(SBMI.GraphicData.Intel) || Condition(SBMI.GraphicData.Nvidia);
        }

        private static bool Condition(JArray Array)
        {
            try
            {
                if (Array == null || Array.Count == 0)
                {
                    return false;
                }

                if (Array[0] is not JObject First || First["Name"] == null || Convert.ToInt32(SHN.Numeral(SSEPPE.Similarity(First["Name"].ToString(), SMMM.GraphicAdapter, SESP.Jaccard), false, false, 0, '0', SECNT.DecimalFraction)) < 75)
                {
                    return false;
                }

                IEnumerable<JObject> Items = Array.OfType<JObject>().Where(Item => Item["Type"]?.ToString() == "Load" && Item["Now"] != null);

                return Items.OfType<JObject>().All(Item => double.TryParse(Item["Now"].ToString(), out double Now) && Now < SMMM.GpuUsage);
            }
            catch
            {
                return false;
            }
        }

        public static bool Performance()
        {
            return Performance(SBMI.GraphicData.Amd) || Performance(SBMI.GraphicData.Intel) || Performance(SBMI.GraphicData.Nvidia);
        }

        private static bool Performance(JArray Array)
        {
            try
            {
                if (Array == null || Array.Count == 0)
                {
                    return false;
                }

                if (Array[0] is not JObject First || First["Name"] == null || Convert.ToInt32(SHN.Numeral(SSEPPE.Similarity(First["Name"].ToString(), SMMM.GraphicAdapter, SESP.Jaccard), false, false, 0, '0', SECNT.DecimalFraction)) < 75)
                {
                    return false;
                }

                return Array.OfType<JObject>().Any(Item => Item["Type"]?.ToString() == "Load" && Item["Now"] != null && double.TryParse(Item["Now"].ToString(), out double Now) && Now >= SMMM.GpuUsage);
            }
            catch
            {
                return false;
            }
        }
    }
}