URL_STUDENT_API="https://raw.githubusercontent.com/polito/api-spec/master/dist/clients/student/openapi.yaml"
FILE_STUDENT_API="spec/openapi_students.yaml"

if [[ ! -e $FILE_STUDENT_API ]]; then
    wget -O $FILE_STUDENT_API $URL_STUDENT_API
fi

openapi-generator-cli generate -g rust -o openapi -i $FILE_STUDENT_API

echo -e " /target/
**/*.rs.bk
Cargo.lock
git_push.sh
README.md
docs
.open* " > openapi/.gitignore