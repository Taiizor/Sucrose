using Newtonsoft.Json;
using System.Collections;
using System.Diagnostics;
using System.Reflection;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSHE = Sucrose.Shared.Space.Helper.Exceptioner;
using SSSISE = Sucrose.Shared.Space.Interface.SerializableException;
using SSSISFD = Sucrose.Shared.Space.Interface.StackFrameData;

namespace Sucrose.Shared.Space.Converter
{
    public class ExceptionConverter : JsonConverter<Exception>
    {
        public override void WriteJson(JsonWriter writer, Exception value, JsonSerializer serializer)
        {
            SSSISE serializableException = ConvertToSerializableException(value);

            serializer.Serialize(writer, serializableException);
        }

        public override Exception ReadJson(JsonReader reader, Type objectType, Exception existingValue, bool hasExistingValue, JsonSerializer serializer)
        {
            SSSISE serializableException = serializer.Deserialize<SSSISE>(reader);

            return ConvertToException(serializableException);
        }

        private List<SSSISFD> ParseStackTrace(Exception exception)
        {
            StackTrace stackTrace = new(exception, true);

            StackFrame[] frames = stackTrace.GetFrames();

            if (frames == null)
            {
                return null;
            }

            List<SSSISFD> frameList = new();

            if (frames.Any())
            {
                foreach (StackFrame frame in frames)
                {
                    frameList.Add(new SSSISFD
                    {
                        FileName = frame.GetFileName(),
                        Method = frame.GetMethod().ToString(),
                        LineNumber = frame.GetFileLineNumber(),
                        ColumnNumber = frame.GetFileColumnNumber()
                    });
                }
            }

            return frameList;
        }

        private Exception ConvertToException(SSSISE serializableException)
        {
            if (serializableException == null)
            {
                return null;
            }

            Type exceptionType = Type.GetType(serializableException.ClassName) ?? typeof(Exception);

            Exception exception = TryCreateInstance(exceptionType, serializableException.Message) ?? TryCreateInstanceWithReflection(exceptionType, serializableException.Message);

            if (exception == null)
            {
                exception = TryCreateInstanceWithConstructor(exceptionType, serializableException.Message);
            }

            exception.HelpLink = serializableException.HelpURL;

            SetExceptionData(exception, serializableException);

            if (serializableException.InnerException != null)
            {
                Exception innerException = ConvertToException(serializableException.InnerException);

                SetField(exception, "_innerException", innerException);
            }

            return exception;
        }

        private SSSISE ConvertToSerializableException(Exception exception)
        {
            if (exception == null)
            {
                return null;
            }

            return new SSSISE
            {
                Data = exception.Data,
                Source = exception.Source,
                Message = exception.Message,
                HResult = exception.HResult,
                HelpURL = exception.HelpLink,
                RawStackTrace = exception.StackTrace,
                StackTrace = ParseStackTrace(exception),
                ClassName = exception.GetType().FullName,
                InnerException = ConvertToSerializableException(exception.InnerException),
                FullMessage = SSSHE.GetMessage(exception, "Unfortunately.", SMMRG.ExceptionSplit)
            };
        }

        private Exception TryCreateInstance(Type exceptionType, string message)
        {
            try
            {
                return (Exception)Activator.CreateInstance(exceptionType, message);
            }
            catch
            {
                return null;
            }
        }

        private void SetField(Exception exception, string fieldName, object value)
        {
            FieldInfo field = typeof(Exception).GetField(fieldName, BindingFlags.NonPublic | BindingFlags.Instance);

            field?.SetValue(exception, value);
        }

        private void SetExceptionData(Exception exception, SSSISE serializableException)
        {
            if (serializableException.Data != null && serializableException.Data.Count > 0)
            {
                if (exception.Data == null)
                {
                    SetField(exception, "_data", new Dictionary<object, object>());
                }

                foreach (DictionaryEntry entry in serializableException.Data)
                {
                    try
                    {
                        exception.Data.Add(JsonConvert.SerializeObject(entry.Key), JsonConvert.SerializeObject(entry.Value));
                    }
                    catch
                    {
                        try
                        {
                            exception.Data.Add(entry.Key, entry.Value);
                        }
                        catch
                        {
                            exception.Data.Add(entry.Key, null);
                        }
                    }
                }
            }
        }

        private Exception TryCreateInstanceWithReflection(Type exceptionType, string message)
        {
            try
            {
                Exception exception = (Exception)Activator.CreateInstance(exceptionType);

                SetField(exception, "_message", message);

                return exception;
            }
            catch
            {
                return null;
            }
        }

        private Exception TryCreateInstanceWithConstructor(Type exceptionType, string message)
        {
            try
            {
                ConstructorInfo constructor = exceptionType.GetConstructor(new[] { typeof(string), typeof(Exception) });

                return (Exception)constructor?.Invoke(new object[] { message, null }) ?? (Exception)Activator.CreateInstance(exceptionType, message) ?? (Exception)Activator.CreateInstance(exceptionType);
            }
            catch
            {
                return null;
            }
        }
    }
}