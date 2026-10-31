# SecuSearch Java SDK

## Directory Structure

- `java-doc`: Java doc files for all the SecuSearch SDK classes
- `lib`: SecuSearch java library
- `sample`: SecuSearch Sample codes

## How to Build & Run the sample codes

1. open a terminal as administrators

2. go to the `sample` folder.

    ```
    > cd sample
    ```

2. compile codes by executing the following gradle command:

    ```
    sample> .\gradlew build
    ```

3. execute the sample codes

    ```
    sample> .\gradlew run
    ```

## Dependencies

-  [Java Native Access(JNA) library](https://github.com/java-native-access/jna): v4.0.0 or later

## Important Notes

- SecuSearch Java SDK supports Java 7 or later.

- SecuSearch Java SDK only supports Windows 32-bit and 64-bit O/S.

- SecuSearch Java SDK always uses the embedded native libraries of `secusearchapi.dll` and `secusearchapi32.dll`
  in the secusearch jar file rather than the DLL files in a system.

- SecuSearch Engine requires an application to have the rights to lock pages in memory.
  Please refer to "2.2. Windows Memory Policy Configuration" in the SeucSearch 3 SDK document.
